package velum

import (
	"context"
	"fmt"
	"net/http"
	"os"

	"github.com/hashicorp/terraform-plugin-sdk/v2/diag"
	"github.com/hashicorp/terraform-plugin-sdk/v2/helper/schema"
)

// Provider возвращает конфигурацию Terraform Provider для Velum
func Provider() *schema.Provider {
	return &schema.Provider{
		Schema: map[string]*schema.Schema{
			"host": {
				Type:        schema.TypeString,
				Optional:    true,
				Description: "Velum API host (e.g., http://localhost:3000). Может быть задана через VELUM_HOST.",
				DefaultFunc: schema.EnvDefaultFunc("VELUM_HOST", "http://localhost:3000"),
			},
			"api_key": {
				Type:        schema.TypeString,
				Optional:    true,
				Sensitive:   true,
				Description: "Velum API key. Может быть задана через VELUM_API_KEY.",
				DefaultFunc: schema.EnvDefaultFunc("VELUM_API_KEY", ""),
			},
		},
		ResourcesMap: map[string]*schema.Resource{
			"velum_project":    resourceProject(),
			"velum_template":   resourceTemplate(),
			"velum_access_key": resourceAccessKey(),
			// TODO: Implement remaining resources
			// "velum_inventory":   resourceInventory(),
			// "velum_repository":  resourceRepository(),
			// "velum_environment": resourceEnvironment(),
			// "velum_schedule":    resourceSchedule(),
		},
		DataSourcesMap: map[string]*schema.Resource{
			"velum_project":   dataSourceProject(),
			"velum_template":  dataSourceTemplate(),
			"velum_inventory": dataSourceInventory(),
		},
		ConfigureContextFunc: providerConfigure,
	}
}

// Config хранит конфигурацию провайдера
type Config struct {
	Host   string
	APIKey string
	Client *http.Client
}

func providerConfigure(ctx context.Context, d *schema.ResourceData) (interface{}, diag.Diagnostics) {
	host := d.Get("host").(string)
	apiKey := d.Get("api_key").(string)

	if host == "" {
		return nil, diag.Errorf("host cannot be empty")
	}

	client := &http.Client{}

	config := Config{
		Host:   host,
		APIKey: apiKey,
		Client: client,
	}

	return &config, nil
}

// buildURL строит полный URL к API endpoint
func (c *Config) buildURL(path string) string {
	return fmt.Sprintf("%s/api%s", c.Host, path)
}

// getAuthHeader возвращает заголовок авторизации
func (c *Config) getAuthHeader() map[string]string {
	headers := map[string]string{
		"Content-Type": "application/json",
	}
	if c.APIKey != "" {
		headers["Authorization"] = "Bearer " + c.APIKey
	}
	return headers
}

// checkError проверяет ошибку и возвращает diag.Diagnostics
func checkError(err error, action string) diag.Diagnostics {
	if err != nil {
		return diag.Errorf("failed to %s: %s", action, err.Error())
	}
	return nil
}

// stringPtr возвращает указатель на строку
func stringPtr(s string) *string {
	return &s
}

// intPtr возвращает указатель на int
func intPtr(i int) *int {
	return &i
}

// int32Ptr возвращает указатель на int32
func int32Ptr(i int32) *int32 {
	return &i
}

// getFileContent читает содержимое файла или возвращает значение
func getFileContent(pathOrContent string) (string, error) {
	if _, err := os.Stat(pathOrContent); err == nil {
		content, err := os.ReadFile(pathOrContent)
		if err != nil {
			return "", err
		}
		return string(content), nil
	}
	return pathOrContent, nil
}
