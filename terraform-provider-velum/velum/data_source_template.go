package velum

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"
	"strconv"

	"github.com/hashicorp/terraform-plugin-sdk/v2/diag"
	"github.com/hashicorp/terraform-plugin-sdk/v2/helper/schema"
)

func dataSourceTemplate() *schema.Resource {
	return &schema.Resource{
		ReadContext: dataSourceTemplateRead,
		Schema: map[string]*schema.Schema{
			"project_id": {
				Type:        schema.TypeInt,
				Required:    true,
				Description: "ID проекта",
			},
			"template_id": {
				Type:        schema.TypeInt,
				Required:    true,
				Description: "ID шаблона",
			},
			"name": {
				Type:        schema.TypeString,
				Computed:    true,
				Description: "Имя шаблона",
			},
			"description": {
				Type:        schema.TypeString,
				Computed:    true,
				Description: "Описание шаблона",
			},
			"playbook": {
				Type:        schema.TypeString,
				Computed:    true,
				Description: "Путь к playbook",
			},
		},
	}
}

func dataSourceTemplateRead(ctx context.Context, d *schema.ResourceData, meta interface{}) diag.Diagnostics {
	config := meta.(*Config)

	projectID := d.Get("project_id").(int)
	templateID := d.Get("template_id").(int)

	req, err := http.NewRequestWithContext(ctx, "GET", config.buildURL(fmt.Sprintf("/project/%d/template/%d", projectID, templateID)), nil)
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
		return diag.Errorf("template with id %d not found in project %d", templateID, projectID)
	}

	if resp.StatusCode != http.StatusOK {
		return diag.Errorf("failed to read template: status %d", resp.StatusCode)
	}

	var template Template
	if err := json.NewDecoder(resp.Body).Decode(&template); err != nil {
		return diag.FromErr(err)
	}

	d.SetId(strconv.Itoa(template.ID))
	d.Set("name", template.Name)
	d.Set("description", template.Description)
	d.Set("playbook", template.Playbook)

	return nil
}
