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

// Project представляет проект Velum
type Project struct {
	ID          int    `json:"id,omitempty"`
	Name        string `json:"name"`
	Description string `json:"description,omitempty"`
}

func resourceProject() *schema.Resource {
	return &schema.Resource{
		CreateContext: resourceProjectCreate,
		ReadContext:   resourceProjectRead,
		UpdateContext: resourceProjectUpdate,
		DeleteContext: resourceProjectDelete,
		Importer: &schema.ResourceImporter{
			StateContext: schema.ImportStatePassthroughContext,
		},
		Schema: map[string]*schema.Schema{
			"name": {
				Type:        schema.TypeString,
				Required:    true,
				Description: "Имя проекта",
			},
			"description": {
				Type:        schema.TypeString,
				Optional:    true,
				Description: "Описание проекта",
			},
		},
	}
}

func resourceProjectCreate(ctx context.Context, d *schema.ResourceData, meta interface{}) diag.Diagnostics {
	config := meta.(*Config)

	project := Project{
		Name:        d.Get("name").(string),
		Description: d.Get("description").(string),
	}

	jsonData, err := json.Marshal(project)
	if err != nil {
		return diag.FromErr(err)
	}

	req, err := http.NewRequestWithContext(ctx, "POST", config.buildURL("/projects/"), bytes.NewReader(jsonData))
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
		return diag.Errorf("failed to create project: %s (status: %d, body: %s)", err, resp.StatusCode, string(body))
	}

	var createdProject Project
	if err := json.NewDecoder(resp.Body).Decode(&createdProject); err != nil {
		return diag.FromErr(err)
	}

	d.SetId(strconv.Itoa(createdProject.ID))

	return resourceProjectRead(ctx, d, meta)
}

func resourceProjectRead(ctx context.Context, d *schema.ResourceData, meta interface{}) diag.Diagnostics {
	config := meta.(*Config)

	id, err := strconv.Atoi(d.Id())
	if err != nil {
		return diag.FromErr(err)
	}

	req, err := http.NewRequestWithContext(ctx, "GET", config.buildURL(fmt.Sprintf("/projects/%d", id)), nil)
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
		return diag.Errorf("failed to read project: status %d", resp.StatusCode)
	}

	var project Project
	if err := json.NewDecoder(resp.Body).Decode(&project); err != nil {
		return diag.FromErr(err)
	}

	d.Set("name", project.Name)
	d.Set("description", project.Description)

	return nil
}

func resourceProjectUpdate(ctx context.Context, d *schema.ResourceData, meta interface{}) diag.Diagnostics {
	config := meta.(*Config)

	id, err := strconv.Atoi(d.Id())
	if err != nil {
		return diag.FromErr(err)
	}

	project := Project{
		Name:        d.Get("name").(string),
		Description: d.Get("description").(string),
	}

	jsonData, err := json.Marshal(project)
	if err != nil {
		return diag.FromErr(err)
	}

	req, err := http.NewRequestWithContext(ctx, "PUT", config.buildURL(fmt.Sprintf("/projects/%d", id)), bytes.NewReader(jsonData))
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
		return diag.Errorf("failed to update project: %s (status: %d, body: %s)", err, resp.StatusCode, string(body))
	}

	return resourceProjectRead(ctx, d, meta)
}

func resourceProjectDelete(ctx context.Context, d *schema.ResourceData, meta interface{}) diag.Diagnostics {
	config := meta.(*Config)

	id, err := strconv.Atoi(d.Id())
	if err != nil {
		return diag.FromErr(err)
	}

	req, err := http.NewRequestWithContext(ctx, "DELETE", config.buildURL(fmt.Sprintf("/projects/%d", id)), nil)
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
		return diag.Errorf("failed to delete project: status %d", resp.StatusCode)
	}

	d.SetId("")
	return nil
}
