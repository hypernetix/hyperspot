package settings

import (
	"context"
	"net/http"

	"github.com/danielgtaylor/huma/v2"
	"github.com/hypernetix/hyperspot/libs/config"
)

type SettingsAPIResponse struct {
	Body Settings `json:"body"`
}

// getSettingHandler retrieves settings for a user and tenant
func getSettingHandler(ctx context.Context, input *struct{}) (*SettingsAPIResponse, error) {
	settings, err := getSettings(ctx)
	if err != nil {
		return nil, err
	}

	return &SettingsAPIResponse{
		Body: *settings,
	}, nil
}

// updateSettingHandler updates settings for a user and tenant
func updateSettingHandler(ctx context.Context, input *struct {
	Body struct {
		Theme    string `json:"theme"`
		Language string `json:"language"`
	} `body:""`
}) (*SettingsAPIResponse, error) {
	settings, errx := getSettings(ctx)
	if errx != nil {
		return nil, errx
	}

	settings.Theme = input.Body.Theme
	settings.Language = input.Body.Language

	if errx := updateSettings(ctx, settings); errx != nil {
		return nil, errx
	}

	return &SettingsAPIResponse{
		Body: *settings,
	}, nil
}

// patchSettingHandler partially updates settings for a user and tenant
func patchSettingHandler(ctx context.Context, input *struct {
	Body struct {
		Theme    *string `json:"theme,omitempty"`
		Language *string `json:"language,omitempty"`
	} `body:""`
}) (*SettingsAPIResponse, error) {
	settings, errx := getSettings(ctx)
	if errx != nil {
		return nil, errx
	}

	// Only update fields that are provided in the request
	if input.Body.Theme != nil {
		settings.Theme = *input.Body.Theme
	}
	if input.Body.Language != nil {
		settings.Language = *input.Body.Language
	}

	if errx := updateSettings(ctx, settings); errx != nil {
		return nil, errx
	}

	return &SettingsAPIResponse{
		Body: *settings,
	}, nil
}

// registerSettingAPIRoutes registers the setting API routes
func registerSettingsAPIRoutes(api huma.API) {
	huma.Register(api, huma.Operation{
		OperationID:     "get-settings",
		Method:          http.MethodGet,
		BodyReadTimeout: config.GetServerTimeout(),
		Path:            "/settings",
		Summary:         "Get user settings",
		Tags:            []string{"Settings"},
	}, getSettingHandler)

	huma.Register(api, huma.Operation{
		OperationID:     "update-settings",
		Method:          http.MethodPost,
		BodyReadTimeout: config.GetServerTimeout(),
		Path:            "/settings",
		Summary:         "Update user settings",
		Tags:            []string{"Settings"},
	}, updateSettingHandler)

	huma.Register(api, huma.Operation{
		OperationID:     "patch-settings",
		Method:          http.MethodPatch,
		BodyReadTimeout: config.GetServerTimeout(),
		Path:            "/settings",
		Summary:         "Partially update user settings",
		Tags:            []string{"Settings"},
	}, patchSettingHandler)
}
