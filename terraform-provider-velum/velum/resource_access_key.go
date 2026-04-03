package velum

import (
	"bytes"
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"
	"strconv"

	"github.com/hashicorp/terraform-plugin-sdk/v2/diag"
	"github.com/hashicorp/terraform-plugin-sdk/v2/helper/schema"
)

// AccessKey представляет ключ доступа Velum
type AccessKey struct {
	ID           int     `json:"id,omitempty"`
	ProjectID    int     `json:"project_id,omitempty"`
	Name         string  `json:"name"`
	Type         string  `json:"type"` // ssh, login_password, none
	LoginPassword string `json:"login_password,omitempty"`
	PrivateKey   string  `json:"private_key,omitempty"`
	LoginUser    string  `json:"login_user,omitempty"`
}

func resourceAccessKey() *schema.Resource {
	return &schema.Resource{
		CreateContext: resourceAccessKeyCreate,
		ReadContext:   resourceAccessKeyRead,
		UpdateContext: resourceAccessKeyUpdate,
		DeleteContext: resourceAccessKeyDelete,
		Importer: &schema.ResourceImporter{
			StateContext: schema.ImportStatePassthroughContext,
		},
		Schema: map[string]*schema.Schema{
			"project_id": {
				Type:        schema.TypeInt,
				Optional:    true,
				Description: "ID проекта (может быть null для глобальных ключей)",
			},
			"name": {
				Type:        schema.TypeString,
				Required:    true,
				Description: "Имя ключа доступа",
			},
			"type": {
				Type:        schema.TypeString,
				Required:    true,
				Description: "Тип ключа: ssh, login_password, none",
			},
			"login_password": {
				Type:        schema.TypeString,
				Optional:    true,
				Sensitive:   true,
				Description: "Пароль для типа login_password",
			},
			"private_key": {
				Type:        schema.TypeString,
				Optional:    true,
				Sensitive:   true,
				Description: "Приватный SSH ключ для типа ssh",
			},
			"login_user": {
				Type:        schema.TypeString,
				Optional:    true,
				Description: "Пользователь для подключения (по умолчанию: root)",
				Default:     "root",
			},
		},
	}
}

func resourceAccessKeyCreate(ctx context.Context, d *schema.ResourceData, meta interface{}) diag.Diagnostics {
	config := meta.(*Config)

	accessKey := AccessKey{
		ProjectID:     d.Get("project_id").(int),
		Name:          d.Get("name").(string),
		Type:          d.Get("type").(string),
		LoginPassword: d.Get("login_password").(string),
		PrivateKey:    d.Get("private_key").(string),
		LoginUser:     d.Get("login_user").(string),
	}

	jsonData, err := json.Marshal(accessKey)
	if err != nil {
		return diag.FromErr(err)
	}

	url := config.buildURL("/access-keys")
	if accessKey.ProjectID > 0 {
		url = config.buildURL(fmt.Sprintf("/project/%d/keys", accessKey.ProjectID))
	}

	req, err := http.NewRequestWithContext(ctx, "POST", url, bytes.NewReader(jsonData))
	if err != nil {
		return diag.FromErr(err)
	}

	for k, v := range config.getAuthHeader() {
		req.Header.Set(k, v)
	}

	resp, err := config.Client.Do(req)
	if err != nil {
		return diag.FromErr(err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusCreated {
		body, _ := io.ReadAll(resp.Body)
		return diag.Errorf("failed to create access key: %s (status: %d, body: %s)", err, resp.StatusCode, string(body))
	}

	var createdKey AccessKey
	if err := json.NewDecoder(resp.Body).Decode(&createdKey); err != nil {
		return diag.FromErr(err)
	}

	d.SetId(strconv.Itoa(createdKey.ID))

	return resourceAccessKeyRead(ctx, d, meta)
}

func resourceAccessKeyRead(ctx context.Context, d *schema.ResourceData, meta interface{}) diag.Diagnostics {
	config := meta.(*Config)

	id, err := strconv.Atoi(d.Id())
	if err != nil {
		return diag.FromErr(err)
	}

	req, err := http.NewRequestWithContext(ctx, "GET", config.buildURL(fmt.Sprintf("/access-keys/%d", id)), nil)
	if err != nil {
		return diag.FromErr(err)
	}

	for k, v := range config.getAuthHeader() {
		req.Header.Set(k, v)
	}

	resp, err := config.Client.Do(req)
	if err != nil {
		return diag.FromErr(err)
	}
	defer resp.Body.Close()

	if resp.StatusCode == http.StatusNotFound {
		d.SetId("")
		return nil
	}

	if resp.StatusCode != http.StatusOK {
		return diag.Errorf("failed to read access key: status %d", resp.StatusCode)
	}

	var key AccessKey
	if err := json.NewDecoder(resp.Body).Decode(&key); err != nil {
		return diag.FromErr(err)
	}

	d.Set("project_id", key.ProjectID)
	d.Set("name", key.Name)
	d.Set("type", key.Type)
	d.Set("login_user", key.LoginUser)
	// Не возвращаем чувствительные данные
	d.Set("login_password", "")
	d.Set("private_key", "")

	return nil
}

func resourceAccessKeyUpdate(ctx context.Context, d *schema.ResourceData, meta interface{}) diag.Diagnostics {
	config := meta.(*Config)

	id, err := strconv.Atoi(d.Id())
	if err != nil {
		return diag.FromErr(err)
	}

	accessKey := AccessKey{
		Name:       d.Get("name").(string),
		Type:       d.Get("type").(string),
		LoginUser:  d.Get("login_user").(string),
	}

	// Обновляем чувствительные данные только если они изменены
	if d.HasChange("login_password") {
		accessKey.LoginPassword = d.Get("login_password").(string)
	}
	if d.HasChange("private_key") {
		accessKey.PrivateKey = d.Get("private_key").(string)
	}

	jsonData, err := json.Marshal(accessKey)
	if err != nil {
		return diag.FromErr(err)
	}

	req, err := http.NewRequestWithContext(ctx, "PUT", config.buildURL(fmt.Sprintf("/access-keys/%d", id)), bytes.NewReader(jsonData))
	if err != nil {
		return diag.FromErr(err)
	}

	for k, v := range config.getAuthHeader() {
		req.Header.Set(k, v)
	}

	resp, err := config.Client.Do(req)
	if err != nil {
		return diag.FromErr(err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		body, _ := io.ReadAll(resp.Body)
		return diag.Errorf("failed to update access key: %s (status: %d, body: %s)", err, resp.StatusCode, string(body))
	}

	return resourceAccessKeyRead(ctx, d, meta)
}

func resourceAccessKeyDelete(ctx context.Context, d *schema.ResourceData, meta interface{}) diag.Diagnostics {
	config := meta.(*Config)

	id, err := strconv.Atoi(d.Id())
	if err != nil {
		return diag.FromErr(err)
	}

	req, err := http.NewRequestWithContext(ctx, "DELETE", config.buildURL(fmt.Sprintf("/access-keys/%d", id)), nil)
	if err != nil {
		return diag.FromErr(err)
	}

	for k, v := range config.getAuthHeader() {
		req.Header.Set(k, v)
	}

	resp, err := config.Client.Do(req)
	if err != nil {
		return diag.FromErr(err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusNoContent && resp.StatusCode != http.StatusOK {
		return diag.Errorf("failed to delete access key: status %d", resp.StatusCode)
	}

	d.SetId("")
	return nil
}
