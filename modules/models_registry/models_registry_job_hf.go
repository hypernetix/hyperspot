package models_registry

import (
	"context"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"
	"net/url"
	"strings"
	"time"

	"github.com/hypernetix/hyperspot/libs/api_client"
	"github.com/hypernetix/hyperspot/libs/db"
	"github.com/hypernetix/hyperspot/modules/job"
	"github.com/hypernetix/hyperspot/modules/llm"
	"gorm.io/gorm"
)

// GitFile represents a file in a repository
type GitFile struct {
	Path string `json:"path"`
	Size int64  `json:"size"`
	Name string `json:"name"`
	Type string `json:"type"` // "file" or "directory"
}

type HuggingFaceRepoType string

const (
	HuggingFaceRepoModels HuggingFaceRepoType = "models"
)

// ListRepoFiles lists all files in a HuggingFace repository
func huggingfaceListRepoFiles(
	ctx context.Context,
	j *job.JobObj,
	repoType HuggingFaceRepoType,
	repoID string,
) ([]GitFile, error) {
	// Extract repo ID from URL
	repoID = strings.TrimPrefix(repoID, "/")
	repoID = strings.TrimSuffix(repoID, "/")

	client := api_client.NewBaseAPIClient("huggingface models registry loader", "https://huggingface.co", 30, 30, true, false)

	j.LogDebug("listing repo files for %s/%s", repoType, repoID)

	// Call HuggingFace API
	apiURL := fmt.Sprintf("/api/models/%s/tree/main", repoID)
	resp, err := client.Get(ctx, apiURL, "")
	if err != nil {
		return nil, fmt.Errorf("failed to fetch repo contents: %w", err)
	}

	if resp.UpstreamResponse.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("API request failed with status %d: %s", resp.UpstreamResponse.StatusCode, string(resp.BodyBytes))
	}

	var files []GitFile
	if err := json.Unmarshal(resp.BodyBytes, &files); err != nil {
		return nil, fmt.Errorf("failed to parse API response: %w", err)
	}

	return files, nil
}

func huggingfaceGetModelSize(ctx context.Context, j *job.JobObj, model *LLMRegistryModel) (int64, error) {
	files, err := huggingfaceListRepoFiles(ctx, j, HuggingFaceRepoModels, model.Name)
	if err != nil {
		return 0, err
	}

	totalSize := int64(0)
	for _, file := range files {
		if file.Type == "file" && file.Size > 1024*1024 {
			// Count only large files
			totalSize += file.Size
		}
		j.LogTrace("model %s has file %s, size %d", model.Name, file.Path, file.Size)
	}

	return totalSize, nil
}

func huggingfaceParseLinkHeader(header string) string {
	if header == "" {
		return ""
	}

	links := strings.Split(header, ",")
	for _, link := range links {
		parts := strings.Split(strings.TrimSpace(link), ";")
		if len(parts) < 2 {
			continue
		}

		// Check if this is the "next" relation
		if strings.Contains(parts[1], `rel="next"`) {
			// Extract URL from <url> and clean it
			urlStr := strings.Trim(strings.TrimSpace(parts[0]), "<>")

			// If it's a full URL, keep only the path and query
			if strings.HasPrefix(urlStr, "https://") {
				if u, err := url.Parse(urlStr); err == nil {
					return u.RequestURI()
				}
			}
			return urlStr
		}
	}
	return ""
}

func huggingfaceModelsUpdate(
	ctx context.Context,
	params *LLMModelRegistryJobParams,
	j *job.JobObj,
	baseProgress float32,
	maxProgress float32,
) (result *LLMModelRegistryJobResultEntry) {
	host := "https://huggingface.co"
	api := api_client.NewBaseAPIClient("huggingface", host, 30, 30, true, false)

	result = &LLMModelRegistryJobResultEntry{
		Updated: 0,
		Created: 0,
		Skipped: 0,
		Errors:  0,
		Total:   0,
	}

	var registryLastUpdate LLMRegistryLastUpdate
	initialSeedingDone := false
	if err := db.DB().Where("registry = ?", LLMRegistryHF).First(&registryLastUpdate).Error; err == nil {
		initialSeedingDone = registryLastUpdate.InitialSeedingDone
	}

	// Get the last update time
	lastSeenModelTimeMs, err := getLatestUpdateTime(LLMRegistryHF)
	if err != nil && !errors.Is(err, gorm.ErrRecordNotFound) {
		result.Error = fmt.Sprintf("failed to get latest update time for %s: %v", LLMRegistryHF, err)
		return result
	}

	if params.ModelsSinceMs > 0 {
		lastSeenModelTimeMs = params.ModelsSinceMs
	}
	lastSeenModelTime := time.UnixMilli(lastSeenModelTimeMs)

	// Determine if we need a full scan
	needInitialSeeding := params.ForceUpdate || lastSeenModelTimeMs == 0 || !initialSeedingDone

	var latestTime, earliestTime, pageMinTime, pageMaxTime time.Time

	baseURL := "/api/models"
	if needInitialSeeding {
		j.LogInfo("Running initial seeding for HuggingFace models registry")
		baseURL += "?sort=createdAt&direction=1&limit=200" // Oldest first
	} else {
		j.LogInfo("Running incremental update for HuggingFace models registry")
		baseURL += "?sort=createdAt&direction=-1&limit=200" // Newest first
	}

	nextURL := baseURL

	retryCount := 0
	retriesMax := 3

	firstPage := true

	for nextURL != "" {
		nextURL = strings.TrimPrefix(nextURL, host)
		resp, err := api.Get(ctx, nextURL, "")
		if err != nil {
			if retryCount >= retriesMax {
				result.Error = fmt.Sprintf("failed to fetch models from %s: %v", nextURL, err)
				return result
			}
			j.LogWarn("failed to fetch models from %s: %s, retrying in %d seconds", nextURL, err.Error(), retryCount)
			time.Sleep(time.Second * time.Duration(retryCount))
			retryCount++
			continue
		}

		if resp.UpstreamResponse.StatusCode != http.StatusOK {
			if retryCount >= retriesMax {
				result.Error = fmt.Sprintf("got unexpected status code %d from %s: %v", resp.UpstreamResponse.StatusCode, nextURL, err)
				return result
			}
			j.LogWarn("got unexpected status code %d from %s: %s, retrying in %d seconds", resp.UpstreamResponse.StatusCode, nextURL, err.Error(), retryCount)
			time.Sleep(time.Second * time.Duration(retryCount))
			retryCount++
			continue
		}

		var models []map[string]interface{}
		if err := json.Unmarshal(resp.BodyBytes, &models); err != nil {
			if retryCount >= retriesMax {
				result.Error = fmt.Sprintf("failed to parse models from %s: %v", nextURL, err)
				return result
			}
			j.LogWarn("failed to parse models from %s: %s, retrying in %d seconds", nextURL, err.Error(), retryCount)
			time.Sleep(time.Second * time.Duration(retryCount))
			retryCount++
			continue
		}

		retryCount = 0

		if needInitialSeeding {
			pageMinTime = parseTime(models[0]["createdAt"])
			pageMaxTime = parseTime(models[len(models)-1]["createdAt"])
		} else {
			pageMinTime = parseTime(models[len(models)-1]["createdAt"])
			pageMaxTime = parseTime(models[0]["createdAt"])
		}

		// Process models
		for _, m := range models {
			if m["_id"] == nil {
				continue
			}

			createdAt := parseTime(m["createdAt"])

			// For incremental update, stop if we've reached already processed models
			if !needInitialSeeding && !lastSeenModelTime.IsZero() && createdAt.Before(lastSeenModelTime) {
				break
			}

			// Track earliest/latest times for progress calculation
			if firstPage {
				if needInitialSeeding {
					earliestTime = createdAt
					latestTime = time.Now().UTC()
				} else {
					earliestTime = lastSeenModelTime
					latestTime = createdAt
				}
				firstPage = false
			}

			model_name := strings.ToLower(fmt.Sprintf("%v", m["modelId"]))
			model_id := strings.ToLower(fmt.Sprintf("%v", m["_id"]))
			likes := parseInt(m["likes"])
			downloads := parseInt(m["downloads"])
			trending_score := parseInt(m["trending_score"])

			tags := []string{}
			if m["tags"] != nil {
				for _, tag := range m["tags"].([]interface{}) {
					tags = append(tags, tag.(string))
				}
			}

			rm := LLMRegistryModel{
				LLMModel: llm.LLMModel{
					Name:      model_name,
					Publisher: strings.Split(model_name, "/")[0],
				},
				DBKey:         fmt.Sprintf("%s:%v", LLMRegistryHF, model_id),
				Registry:      LLMRegistryHF,
				ID:            model_id,
				ModelID:       model_id,
				Likes:         likes,
				TrendingScore: trending_score,
				Downloads:     downloads,
				CreatedAtMs:   createdAt.UnixMilli(), // FIXME: what about timezone? UTC?
				Tags:          strings.Join(tags, ","),
				URL:           fmt.Sprintf("https://huggingface.co/%v", m["id"]),
			}

			status, err := updateLLMRegistryModel(&rm)
			if err != nil {
				result.Error = fmt.Sprintf("failed to update model %s: %v", rm.DBKey, err)
				return result
			}

			if status == LLMModelRegistryUpdateStatusUpdated {
				result.Updated++
			} else if status == LLMModelRegistryUpdateStatusNew {
				result.Created++
			} else if status == LLMModelRegistryUpdateStatusSkipped {
				result.Skipped++
			} else if status == LLMModelRegistryUpdateStatusError {
				result.Errors++
			}

			if params.ForceUpdate || status == LLMModelRegistryUpdateStatusNew {
				if params.GetInfoForAll || rm.Downloads > 50 || rm.Likes > 50 || rm.TrendingScore > 80 {
					rm.Size, err = huggingfaceGetModelSize(ctx, j, &rm)
					if err != nil {
						j.LogWarn("failed to get model size for %s: %s", rm.DBKey, err.Error())
						rm.Size = 0
						result.Errors++
					}
				}
			}

			msg := fmt.Sprintf("HuggingFace model info fetch: %s, Downloads: %d, Likes: %d, Trend: %d",
				rm.Name, rm.Downloads, rm.Likes, rm.TrendingScore)
			if rm.Size > 0 {
				msg += fmt.Sprintf(", Size %.1f GBytes", float64(rm.Size)/(1024.0*1024.0*1024.0))
			} else {
				msg += ", Size unknown"
			}
			j.LogDebug(msg)

			result.Total++
		}

		// Get next page URL from Link header
		nextURL = huggingfaceParseLinkHeader(resp.UpstreamResponse.Header.Get("Link"))

		// Calculate and send progress
		if !earliestTime.IsZero() && !latestTime.IsZero() && !pageMinTime.IsZero() && !pageMaxTime.IsZero() {
			timeRange := latestTime.Sub(earliestTime)
			if timeRange > 0 {
				var progressTime time.Time
				if needInitialSeeding {
					progressTime = pageMinTime
				} else {
					progressTime = pageMaxTime
				}
				progressVal := baseProgress + float32(progressTime.Sub(earliestTime))/float32(timeRange)*(maxProgress-baseProgress)

				j.LogInfo("Progress: %d (%.1f%%), %s, %s", result.Total, progressVal, progressTime, earliestTime)

				if progressVal > 0 && progressVal <= 100 {
					if err := j.SetProgress(ctx, progressVal); err != nil {
						j.LogWarn("Failed to set progress: %v", err)
					}
				}
			}
		}
	}

	if needInitialSeeding {
		if err := db.DB().Model(&LLMRegistryLastUpdate{}).
			Where("registry = ?", LLMRegistryHF).
			Updates(map[string]interface{}{
				"initial_seeding_done": true,
				"updated":              time.Now().UTC(),
			}); err != nil {
			result.Error = fmt.Sprintf("failed to update registry last update: %v", err)
			return result
		}
	}

	return result
}
