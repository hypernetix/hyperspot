package models_registry

import (
	"context"
	"errors"
	"fmt"
	"reflect"
	"strconv"
	"time"

	"github.com/google/uuid"
	"github.com/hypernetix/hyperspot/libs/db"
	"github.com/hypernetix/hyperspot/libs/errorx"
	"github.com/hypernetix/hyperspot/libs/logging"
	"github.com/hypernetix/hyperspot/libs/utils"
	"github.com/hypernetix/hyperspot/modules/job"
	"gorm.io/gorm"
)

// Define job queue and group for models registry jobs
var jobQueueModelsRegistry = job.JobQueueName("models_registry")

var jobGroupModelsRegistry = &job.JobGroup{
	Name:        "llm_model_registry",
	Description: "LLM model registry jobs",
	Queue:       &job.JobQueueConfig{Capacity: 2, Name: jobQueueModelsRegistry},
}

// Store job type for later use
var modelsRegistryUpdateJobType *job.JobType

type LLMModelRegistryUpdateStatus int

const (
	LLMModelRegistryUpdateStatusNew LLMModelRegistryUpdateStatus = iota
	LLMModelRegistryUpdateStatusUpdated
	LLMModelRegistryUpdateStatusSkipped
	LLMModelRegistryUpdateStatusError
)

// LLMModelJob represents a LLM model management job
// All parameters are optional and will use sensible defaults if not provided
type LLMModelRegistryJobParams struct {
	ForceUpdate   bool   `json:"force_update,omitempty" doc:"If true, force update all models"`
	Registry      string `json:"registry,omitempty" doc:"Registry to update (defaults to HuggingFace if not specified)"`
	ModelsSinceMs int64  `json:"models_since,omitempty" doc:"If provided, only models newer than this date will be updated. The date is unix timestamp in milliseconds"`
	GetInfoForAll bool   `json:"get_info_for_all,omitempty" doc:"If true, get info for all models, otherwise only get info for more or less popular models"`
}

type LLMModelRegistryJobResultEntry struct {
	JobID    uuid.UUID `json:"job_id" readOnly:"true"`
	Registry string    `json:"registry"`
	Updated  int       `json:"updated"`
	Created  int       `json:"created"`
	Skipped  int       `json:"skipped"`
	Errors   int       `json:"errors"`
	Total    int       `json:"total"`
	Error    string    `json:"error"`
}

type LLMModelRegistryJobResult struct {
	Entries []*LLMModelRegistryJobResultEntry `gorm:"type:jsonb" json:"entries"`
}

func parseInt(val interface{}) int {
	switch v := val.(type) {
	case float32:
		return int(v)
	case float64:
		return int(v)
	case int:
		return v
	case int64:
		return int(v)
	case string:
		if i, err := strconv.Atoi(v); err == nil {
			return i
		}
	}
	return 0
}

func parseTime(val interface{}) time.Time {
	switch v := val.(type) {
	case string:
		// Try multiple time formats
		formats := []string{
			time.RFC3339,
			"2006-01-02 15:04:05",
			"2006-01-02T15:04:05",
			"2006-01-02T15:04:05Z07:00",
			"2006-01-02T15:04:05.999999999Z07:00",
		}

		for _, format := range formats {
			t, err := time.Parse(format, v)
			if err == nil {
				return t
			}
		}

		// Try parsing as Unix timestamp
		if i, err := strconv.ParseInt(v, 10, 64); err == nil {
			// Check if it's milliseconds or seconds
			if i > 1000000000000 { // Likely milliseconds
				return time.UnixMilli(i)
			} else { // Likely seconds
				return time.Unix(i, 0)
			}
		}
	case time.Time:
		return v
	case *time.Time:
		if v != nil {
			return *v
		}
	}
	return time.Time{}
}

func updateLLMRegistryModel(rm *LLMRegistryModel) (status LLMModelRegistryUpdateStatus, err error) {
	rm.UpdateDetailsFromName()
	var existing LLMRegistryModel
	if err := db.DB().Where("db_key = ?", rm.DBKey).First(&existing).Error; err != nil {
		if errors.Is(err, gorm.ErrRecordNotFound) {
			logging.Debug("Model registry: creating new model: %s %s", LLMRegistryHF, rm.Name)
			if err := db.DB().Create(&rm).Error; err != nil {
				return LLMModelRegistryUpdateStatusError, err
			}
			return LLMModelRegistryUpdateStatusNew, nil
		} else {
			return LLMModelRegistryUpdateStatusError, err
		}
	} else {
		if reflect.DeepEqual(existing, rm) {
			// Entry exists and is identical; skip creation/update.
			return LLMModelRegistryUpdateStatusSkipped, nil
		} else {
			// Entry exists but differs; update the existing record.
			if err := db.DB().Model(&existing).Updates(rm).Error; err != nil {
				return LLMModelRegistryUpdateStatusError, err
			}
			return LLMModelRegistryUpdateStatusUpdated, nil
		}
	}
}

// getLatestUpdateTime retrieves the latest update time for a registry
func getLatestUpdateTime(registry string) (int64, error) {
	var lastUpdateMs int64
	err := db.DB().Model(&LLMRegistryModel{}).
		Where("registry = ?", registry).
		Order("created_at_ms DESC").
		Limit(1).
		Pluck("created_at_ms", &lastUpdateMs).Error
	return lastUpdateMs, err
}

// LLMModelRegistryJobWorker performs work for LLM model registry jobs.
func LLMModelRegistryJobWorker(ctx context.Context, job *job.JobObj) errorx.Error {
	params, ok := job.GetParamsPtr().(*LLMModelRegistryJobParams)
	if !ok {
		return errorx.NewErrInternalServerError("invalid job parameters type")
	}

	err := job.SetProgress(ctx, 0)
	if err != nil {
		return err
	}

	// FIXME:
	// Need to think if we need to have separate jobs for separate models registries
	// or we want to have single job for all. Maybe several jobs are better from
	// scalability perspective.

	// For now, let's have single job for all registries.

	hfStatus := huggingfaceModelsUpdate(ctx, params, job, 0, 50)
	if hfStatus.Error != "" {
		msg := fmt.Sprintf("Failed to update HuggingFace models registry: %v", hfStatus.Error)
		logging.Error(msg)
		return errorx.NewErrInternalServerError(msg)
	}

	/*
		ollamaUpdated, ollamaTotal, ollamaErr := ollamaModelsUpdate(ctx, params.ForceUpdate, job, 0, 50, progressCh)
		if ollamaErr != nil {
			msg := fmt.Sprintf("Failed to update Ollama models registry: %v", ollamaErr)
			logging.Error(msg)
			return errorx.NewErrInternalServerError(msg)
		}
	*/

	errx := job.SetProgress(ctx, 100)
	if errx != nil {
		return errx
	}

	result := LLMModelRegistryJobResult{
		Entries: []*LLMModelRegistryJobResultEntry{
			hfStatus,
			// ollamaStatus,
		},
	}
	errx = job.SetResult(ctx, &result)
	if errx != nil {
		return errx
	}
	return nil
}

// LLMModelRegistryJobParamsValidation initializes the model registry job and validates parameters
func LLMModelRegistryJobParamsValidation(ctx context.Context, j *job.JobObj) errorx.Error {
	paramsPtr := j.GetParamsPtr()
	if paramsPtr == nil {
		return errorx.NewErrInternalServerError("invalid job parameters; parameters are nil")
	}

	jobParams, ok := paramsPtr.(*LLMModelRegistryJobParams)
	if !ok {
		return errorx.NewErrInternalServerError("invalid job parameters type; expected *LLMModelRegistryJobParams")
	}

	// Set defaults for optional parameters
	if jobParams.Registry == "" {
		j.LogDebug("Registry not specified, defaulting to HuggingFace")
		jobParams.Registry = string(LLMRegistryHF)
	}

	// No need to set defaults for other parameters as they have sensible zero values:
	// ForceUpdate and GetInfoForAll default to false
	// ModelsSince defaults to zero time (IsZero() will be true)

	j.LogDebug("LLMModelRegistryJobInit: %+v", jobParams)

	return nil
}

// initModelsRegistryJobs initializes all model registry jobs
func initModelsRegistryJobs() error {
	// Register job queue with max 2 parallel executors
	_, err := job.JERegisterJobQueue(&job.JobQueueConfig{Capacity: 2, Name: jobQueueModelsRegistry})
	if err != nil {
		logging.Error("Failed to register job queue: %v", err)
		return fmt.Errorf("failed to register job queue: %w", err)
	}

	// Register job group
	job.RegisterJobGroup(jobGroupModelsRegistry)

	// Create default job parameters
	jobParams := &LLMModelRegistryJobParams{}
	utils.InitStructWithDefaults(jobParams)

	// Register job type
	modelsRegistryUpdateJobType = job.RegisterJobType(
		job.JobTypeParams{
			Group:                          jobGroupModelsRegistry,
			Name:                           "update",
			Description:                    "Update models registry",
			Params:                         jobParams,
			WorkerParamsValidationCallback: LLMModelRegistryJobParamsValidation,
			WorkerExecutionCallback:        LLMModelRegistryJobWorker,
			WorkerStateUpdateCallback:      nil,
			Timeout:                        time.Hour * 10,
			MaxRetries:                     5,
			RetryDelay:                     time.Second * 30,
			WorkerIsSuspendable:            false,
		},
	)

	return nil
}
