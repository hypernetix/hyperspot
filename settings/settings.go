package settings

import (
	"context"

	"github.com/google/uuid"
	"github.com/hypernetix/hyperspot/libs/auth"
	"github.com/hypernetix/hyperspot/libs/core"
	"github.com/hypernetix/hyperspot/libs/db"
	"github.com/hypernetix/hyperspot/libs/errorx"
	"github.com/hypernetix/hyperspot/libs/logging"
	"github.com/hypernetix/hyperspot/libs/utils"
	"gorm.io/gorm"
)

var mu utils.DebugMutex

// Setting represents user settings
type Settings struct {
	Theme    string    `json:"theme" db:"theme" default:""`
	Language string    `json:"language" db:"language" default:""`
	UserID   uuid.UUID `json:"-" db:"user_id,primaryKey"`
	TenantID uuid.UUID `json:"-" db:"tenant_id,primaryKey"`
}

func getSettings(ctx context.Context) (*Settings, errorx.Error) {
	var settings Settings
	if err := db.DB().Where("tenant_id = ? AND user_id = ?", auth.GetTenantID(), auth.GetUserID()).First(&settings).Error; err != nil {
		if err == gorm.ErrRecordNotFound {
			settings.UserID = auth.GetUserID()
			settings.TenantID = auth.GetTenantID()
			return &settings, nil
		}
		return nil, errorx.NewErrInternalServerError("%s", err.Error())
	}
	return &settings, nil
}

func updateSettings(ctx context.Context, settings *Settings) errorx.Error {
	settings.UserID = auth.GetUserID()
	settings.TenantID = auth.GetTenantID()

	logging.Debug("Updating settings for user %s in tenant %s", settings.UserID, settings.TenantID)

	mu.Lock()
	defer mu.Unlock()

	// Check if the record exists
	var count int64
	if err := db.DB().Model(&Settings{}).Where("user_id = ? AND tenant_id = ?", settings.UserID, settings.TenantID).Count(&count).Error; err != nil {
		return errorx.NewErrInternalServerError("%s", err.Error())
	}

	// If record doesn't exist, create it; otherwise, update it
	if count == 0 {
		if err := db.DB().Create(settings).Error; err != nil {
			return errorx.NewErrInternalServerError("Failed to create settings: %s", err.Error())
		}
	} else {
		if err := db.DB().Where("user_id = ? AND tenant_id = ?", settings.UserID, settings.TenantID).Updates(settings).Error; err != nil {
			return errorx.NewErrInternalServerError("Failed to update settings: %s", err.Error())
		}
	}
	return nil
}

// InitModule initializes the setting module
func InitModule() {
	core.RegisterModule(&core.Module{
		Name: "settings",
		Migrations: []interface{}{
			&Settings{},
		},
		InitAPIRoutes: registerSettingsAPIRoutes,
		InitMain:      nil, // No background jobs for settings
	})
}
