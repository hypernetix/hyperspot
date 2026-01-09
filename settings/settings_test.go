package settings

import (
	"context"
	"testing"

	"github.com/hypernetix/hyperspot/libs/auth"
	"github.com/hypernetix/hyperspot/libs/db"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
	"gorm.io/gorm"
)

// setupTestDB initializes an in-memory SQLite database and auto-migrates the Settings schema.
func setupTestDB(t *testing.T) *gorm.DB {
	t.Helper()
	testDB, err := db.InitInMemorySQLite(nil)
	require.NoError(t, err, "Failed to connect to test DB")
	db.SetDB(testDB)
	err = db.SafeAutoMigrate(testDB, &Settings{})
	require.NoError(t, err, "Failed to migrate test database")
	return testDB
}

// TestWriteAndReadSettings tests writing a setting to the database and reading it back
func TestWriteAndReadSettings(t *testing.T) {
	// Setup test database
	testDB := setupTestDB(t)
	defer func() {
		sqlDB, _ := testDB.DB()
		sqlDB.Close()
	}()

	// Create a context
	ctx := context.Background()

	// Test data
	theme := "dark"
	language := "fr"

	// 1. First, verify no settings exist yet
	initialSettings, err := getSettings(ctx)
	require.NoError(t, err, "Failed to get initial settings")
	assert.Equal(t, "", initialSettings.Theme, "Theme should be empty initially")
	assert.Equal(t, "", initialSettings.Language, "Language should be empty initially")
	assert.Equal(t, auth.GetUserID(), initialSettings.UserID, "User ID should match")
	assert.Equal(t, auth.GetTenantID(), initialSettings.TenantID, "Tenant ID should match")

	// 2. Update settings
	initialSettings.Theme = theme
	initialSettings.Language = language

	err = updateSettings(ctx, initialSettings)
	require.NoError(t, err, "Failed to update settings")

	// 3. Read settings back and verify they match
	updatedSettings, err := getSettings(ctx)
	require.NoError(t, err, "Failed to get updated settings")
	assert.Equal(t, theme, updatedSettings.Theme, "Theme should match what was set")
	assert.Equal(t, language, updatedSettings.Language, "Language should match what was set")
	assert.Equal(t, auth.GetUserID(), updatedSettings.UserID, "User ID should match")
	assert.Equal(t, auth.GetTenantID(), updatedSettings.TenantID, "Tenant ID should match")

	// 4. Change settings again
	newTheme := "light"
	newLanguage := "en"

	updatedSettings.Theme = newTheme
	updatedSettings.Language = newLanguage

	err = updateSettings(ctx, updatedSettings)
	require.NoError(t, err, "Failed to update settings again")

	// 5. Read settings back and verify they match the new values
	finalSettings, err := getSettings(ctx)
	require.NoError(t, err, "Failed to get final settings")
	assert.Equal(t, newTheme, finalSettings.Theme, "Theme should match the new value")
	assert.Equal(t, newLanguage, finalSettings.Language, "Language should match the new value")
	assert.Equal(t, auth.GetUserID(), finalSettings.UserID, "User ID should match")
	assert.Equal(t, auth.GetTenantID(), finalSettings.TenantID, "Tenant ID should match")

	// 6. Verify direct database query also shows the updated values
	var dbSettings Settings
	result := testDB.Where("tenant_id = ? AND user_id = ?", auth.GetTenantID(), auth.GetUserID()).First(&dbSettings)
	require.NoError(t, result.Error, "Failed to query settings directly from DB")
	assert.Equal(t, newTheme, dbSettings.Theme, "DB theme should match")
	assert.Equal(t, newLanguage, dbSettings.Language, "DB language should match")
	assert.Equal(t, auth.GetUserID(), dbSettings.UserID, "DB user ID should match")
	assert.Equal(t, auth.GetTenantID(), dbSettings.TenantID, "DB tenant ID should match")
}
