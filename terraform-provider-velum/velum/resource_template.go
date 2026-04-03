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

// Template представляет шаблон задачи Velum
type Template struct {
	ID          int    `json:"id,omitempty"`
	ProjectID   int    `json:"project_id"`
	Name        string `json:"name"`
	Description string `json:"description,omitempty"`
	Playbook    string `json:"playbook"`
	InventoryID *int   `json:"inventory_id,omitempty"`
	RepositoryID *int  `json:"repository_id,omitempty"`
	EnvironmentID *int `json:"environment_id,omitempty"`
}

func resourceTemplate() *schema.Resource {
	return &schema.Resource{
		CreateContext: resourceTemplateCreate,
		ReadContext:   resourceTemplateRead,
		UpdateContext: resourceTemplateUpdate,
		DeleteContext: resourceTemplateDelete,
		Importer: &schema.ResourceImporter{
			StateContext: schema.ImportStatePassthroughContext,
		},
		Schema: map[string]*schema.Schema{
			"project_id": {
				Type:        schema.TypeInt,
				Required:    true,
				Description: "ID проекта",
			},
			"name": {
				Type:        schema.TypeString,
				Required:    true,
				Description: "Имя шаблона",
			},
			"description": {
				Type:        schema.TypeString,
				Optional:    true,
				Description: "Описание шаблона",
			},
			"playbook": {
				Type:        schema.TypeString,
				Required:    true,
				Description: "Путь к playbook (e.g., site.yml)",
			},
			"inventory_id": {
				Type:        schema.TypeInt,
				Optional:    true,
				Description: "ID inventory",
			},
			"repository_id": {
				Type:        schema.TypeInt,
				Optional:    true,
				Description: "ID repository",
			},
			"environment_id": {
				Type:        schema.TypeInt,
				Optional:    true,
				Description: "ID environment",
			},
		},
	}
}

func resourceTemplateCreate(ctx context.Context, d *schema.ResourceData, meta interface{}) diag.Diagnostics {
	config := meta.(*Config)

	template := Template{
		ProjectID:     d.Get("project_id").(int),
		Name:          d.Get("name").(string),
		Description:   d.Get("description").(string),
		Playbook:      d.Get("playbook").(string),
		InventoryID:   getIntPtr(d, "inventory_id"),
		RepositoryID:  getIntPtr(d, "repository_id"),
		EnvironmentID: getIntPtr(d, "environment_id"),
	}

	jsonData, err := json.Marshal(template)
	if err != nil {
		return diag.FromErr(err)
	}

	req, err := http.NewRequestWithContext(ctx, "POST", config.buildURL(fmt.Sprintf("/project/%d/templates", template.ProjectID)), bytes.NewReader(jsonData))
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
		return diag.Errorf("failed to create template: %s (status: %d, body: %s)", err, resp.StatusCode, string(body))
	}

	var createdTemplate Template
	if err := json.NewDecoder(resp.Body).Decode(&createdTemplate); err != nil {
		return diag.FromErr(err)
	}

	d.SetId(strconv.Itoa(createdTemplate.ID))

	return resourceTemplateRead(ctx, d, meta)
}

func resourceTemplateRead(ctx context.Context, d *schema.ResourceData, meta interface{}) diag.Diagnostics {
	config := meta.(*Config)

	id, err := strconv.Atoi(d.Id())
	if err != nil {
		return diag.FromErr(err)
	}

	// Получаем project_id из state
	projectID := d.Get("project_id").(int)

	req, err := http.NewRequestWithContext(ctx, "GET", config.buildURL(fmt.Sprintf("/project/%d/template/%d", projectID, id)), nil)
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
		return diag.Errorf("failed to read template: status %d", resp.StatusCode)
	}

	var template Template
	if err := json.NewDecoder(resp.Body).Decode(&template); err != nil {
		return diag.FromErr(err)
	}

	d.Set("project_id", template.ProjectID)
	d.Set("name", template.Name)
	d.Set("description", template.Description)
	d.Set("playbook", template.Playbook)
	d.Set("inventory_id", template.InventoryID)
	d.Set("repository_id", template.RepositoryID)
	d.Set("environment_id", template.EnvironmentID)

	return nil
}

func resourceTemplateUpdate(ctx context.Context, d *schema.ResourceData, meta interface{}) diag.Diagnostics {
	config := meta.(*Config)

	id, err := strconv.Atoi(d.Id())
	if err != nil {
		return diag.FromErr(err)
	}

	projectID := d.Get("project_id").(int)

	template := Template{
		ProjectID:     projectID,
		Name:          d.Get("name").(string),
		Description:   d.Get("description").(string),
		Playbook:      d.Get("playbook").(string),
		InventoryID:   getIntPtr(d, "inventory_id"),
		RepositoryID:  getIntPtr(d, "repository_id"),
		EnvironmentID: getIntPtr(d, "environment_id"),
	}

	jsonData, err := json.Marshal(template)
	if err != nil {
		return diag.FromErr(err)
	}

	req, err := http.NewRequestWithContext(ctx, "PUT", config.buildURL(fmt.Sprintf("/project/%d/template/%d", projectID, id)), bytes.NewReader(jsonData))
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
		return diag.Errorf("failed to update template: %s (status: %d, body: %s)", err, resp.StatusCode, string(body))
	}

	return resourceTemplateRead(ctx, d, meta)
}

func resourceTemplateDelete(ctx context.Context, d *schema.ResourceData, meta interface{}) diag.Diagnostics {
	config := meta.(*Config)

	id, err := strconv.Atoi(d.Id())
	if err != nil {
		return diag.FromErr(err)
	}

	projectID := d.Get("project_id").(int)

	req, err := http.NewRequestWithContext(ctx, "DELETE", config.buildURL(fmt.Sprintf("/project/%d/template/%d", projectID, id)), nil)
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
		return diag.Errorf("failed to delete template: status %d", resp.StatusCode)
	}

	d.SetId("")
	return nil
}

func getIntPtr(d *schema.ResourceData, key string) *int {
	if v, ok := d.GetOk(key); ok {
		val := v.(int)
		return &val
	}
	return nil
}
