package models_registry

import (
	"context"
	"fmt"
	"net/http"

	"github.com/danielgtaylor/huma/v2"
	"github.com/hypernetix/hyperspot/libs/api"
)

type ListModelsRegistryAPIRequest struct {
	api.PageAPIRequest
	Query string `query:"query" doc:"Query to filter models like: model like 'llama' and (downloads gt 1000 or likes gt 100)"`
}

// CountModelsRegistryAPIRequest represents the API request payload for counting models.
type CountModelsRegistryAPIRequest struct {
	Query    string `query:"query" doc:"Query to filter models like: model like 'llama' and (downloads gt 1000 or likes gt 100)"`
	Registry string `query:"registry" doc:"Filter by registry name (e.g., huggingface, ollama, cortex)"`
}

// CountModelsRegistryAPIResponse represents the API response payload for counting models.
type CountModelsRegistryAPIResponse struct {
	Body struct {
		Count int `json:"count"`
	} `json:"body"`
}

// ListModelsRegistryAPIResponse represents the API response payload.
type ListModelsRegistryAPIResponse struct {
	Body struct {
		api.PageAPIResponse
		Models []LLMRegistryModel `json:"models"`
	} `json:"body"`
}

// GetModelsRegistry handles GET /models/registry.
// It supports optional query parameters for filtering and paging.
func ListModelsRegistry(ctx context.Context, input *ListModelsRegistryAPIRequest) (*ListModelsRegistryAPIResponse, error) {
	ModelsRegistryResponse := &ListModelsRegistryAPIResponse{}

	if input.Order == "" {
		input.Order = "-downloads"
	}

	err := api.PageAPIInitResponse(&input.PageAPIRequest, &ModelsRegistryResponse.Body.PageAPIResponse)
	if err != nil {
		return nil, huma.Error400BadRequest(err.Error())
	}

	// Query the registry models using the built filters.
	models, err := GetModelsFromRegistry(ctx, &input.PageAPIRequest, input.Query)
	if err != nil {
		return nil, huma.Error500InternalServerError(fmt.Sprintf("Failed to get models registry: %v", err))
	}

	ModelsRegistryResponse.Body.Models = models
	ModelsRegistryResponse.Body.Total = len(models)

	return ModelsRegistryResponse, nil
}

// CountModelsRegistry handles GET /models_registry/count.
// It returns the count of models matching the given filters.
func CountModelsRegistry(ctx context.Context, input *CountModelsRegistryAPIRequest) (*CountModelsRegistryAPIResponse, error) {
	response := &CountModelsRegistryAPIResponse{}

	// Build the query string safely
	query := input.Query
	// Add registry filter if provided
	if input.Registry != "" {
		if input.Registry != LLMRegistryHF && input.Registry != LLMRegistryOllama && input.Registry != LLMRegistryCortex {
			return nil, huma.Error400BadRequest("Invalid registry")
		}

		if query != "" {
			query = fmt.Sprintf("registry eq %s and (%s)", input.Registry, query)
		} else {
			query = fmt.Sprintf("registry eq %s", input.Registry)
		}
	}

	// Count the models
	count, err := CountModelsInRegistry(ctx, query)
	if err != nil {
		return nil, huma.Error500InternalServerError(fmt.Sprintf("Failed to count models in registry: %v", err))
	}

	response.Body.Count = count
	return response, nil
}

// RegisterModelsRegistryRoutes registers the models registry route with the API.
func initModelsRegistryAPIRoutes(humaApi huma.API) {
	api.RegisterEndpoint(humaApi, huma.Operation{
		OperationID: "list-models-registry",
		Method:      http.MethodGet,
		Path:        "/models_registry",
		Summary:     "List all models from the registry with optional filters",
		Tags:        []string{"Global Models Registry"},
	}, ListModelsRegistry)

	api.RegisterEndpoint(humaApi, huma.Operation{
		OperationID: "count-models-registry",
		Method:      http.MethodGet,
		Path:        "/models_registry/count",
		Summary:     "Count models in the registry with optional filters",
		Tags:        []string{"Global Models Registry"},
	}, CountModelsRegistry)
}
