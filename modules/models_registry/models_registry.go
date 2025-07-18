package models_registry

import (
	"context"
	"fmt"
	"strings"

	"github.com/google/uuid"
	"github.com/hypernetix/hyperspot/libs/api"
	"github.com/hypernetix/hyperspot/libs/core"
	"github.com/hypernetix/hyperspot/libs/db"
	"github.com/hypernetix/hyperspot/libs/errorx"
	"github.com/hypernetix/hyperspot/libs/logging"
	"github.com/hypernetix/hyperspot/libs/orm"
	"github.com/hypernetix/hyperspot/modules/llm"
)

const (
	LLMRegistryHF     = "huggingface"
	LLMRegistryOllama = "ollama"
	LLMRegistryCortex = "cortex"
)

type LLMRegistryLastUpdate struct {
	Registry           string `json:"registry"`
	InitialSeedingDone bool   `json:"initial_seeding_done"`
	UpdatedAtMs        int64  `json:"updated_at" gorm:"index"`
}

type LLMRegistryModel struct {
	llm.LLMModel  `json:",inline"`
	DBKey         string `json:"-" gorm:"unique"`
	Registry      string `json:"registry"`
	ID            string `json:"id" gorm:"index"`
	ModelID       string `json:"model_id" gorm:"index"`
	Likes         int    `json:"likes"`
	TrendingScore int    `json:"trending_score"`
	Downloads     int    `json:"downloads" gorm:"index"`
	CreatedAtMs   int64  `json:"created_at" gorm:"index"`
	Tags          string `json:"tags" doc:"Comma separated list of tags"`
	URL           string `json:"url"`
}

// allowedRegistryFields defines the fields that can be used in queries for registry models
var allowedRegistryFields = []string{
	"registry",
	"name",
	"description",
	"publisher",
	"architecture",
	"quantization",
	"streaming",
	"instructed",
	"coding",
	"tooling",
	"is_mlx",
	"is_gguf",
	"tags",
	"size",
	"likes",
	"trending_score",
	"downloads",
	"created_at_ms",
}

func GetModelsFromRegistry(
	ctx context.Context,
	pageRequest *api.PageAPIRequest,
	query string,
) ([]LLMRegistryModel, errorx.Error) {
	query = strings.ToLower(query)

	dbQuery, errx := orm.GetBaseQuery(&LLMRegistryModel{}, uuid.Nil, uuid.Nil, pageRequest)
	if errx != nil {
		return nil, errx
	}

	dbQuery, errx = orm.QueryToGorm(query, allowedRegistryFields, dbQuery)
	if errx != nil {
		return nil, errx
	}

	var models []LLMRegistryModel
	err := dbQuery.Find(&models).Error
	if err != nil {
		return nil, errorx.NewErrInternalServerError("Failed to list models: " + err.Error())
	}
	logging.Debug("Found %d models in registry", len(models))
	return models, nil
}

// CountModelsInRegistry counts the number of models in the registry that match the given query
func CountModelsInRegistry(ctx context.Context, query string) (int, errorx.Error) {
	query = strings.ToLower(query)

	// Start with a base query on the model
	dbQuery := db.DB().Model(&LLMRegistryModel{})

	// Apply filters if query is provided
	if query != "" {
		// Use the QueryToGorm helper to convert the query string to GORM conditions
		filtered, errx := orm.QueryToGorm(query, allowedRegistryFields, dbQuery)
		if errx != nil {
			return 0, errx
		}
		dbQuery = filtered
	}

	// Count the models using GORM's Count method
	var count int64
	err := dbQuery.Count(&count).Error
	if err != nil {
		return 0, errorx.NewErrInternalServerError("Failed to count models: " + err.Error())
	}

	logging.Debug("Counted %d models in registry matching query: %s", count, query)
	return int(count), nil
}

func initModelsRegistry() error {
	if err := initModelsRegistryJobs(); err != nil {
		return fmt.Errorf("failed to init models registry jobs: %w", err)
	}
	return nil
}

func InitModule() {
	core.RegisterModule(&core.Module{
		Migrations: []interface{}{
			&LLMRegistryModel{},
			//			&LLMModelRegistryJobParams{},
			//			&LLMModelRegistryJobResult{},
			// &LLMRegistryLastUpdate{},
		},
		InitMain:      initModelsRegistry,
		InitAPIRoutes: initModelsRegistryAPIRoutes,
		Name:          "models_registry",
	})
}
