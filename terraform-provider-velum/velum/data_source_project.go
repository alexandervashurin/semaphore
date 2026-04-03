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

func dataSourceProject() *schema.Resource {
	return &schema.Resource{
		ReadContext: dataSourceProjectRead,
		Schema: map[string]*schema.Schema{
			"project_id": {
				Type:        schema.TypeInt,
				Required:    true,
				Description: "ID проекта",
			},
			"name": {
				Type:        schema.TypeString,
				Computed:    true,
				Description: "Имя проекта",
			},
			"description": {
				Type:        schema.TypeString,
				Computed:    true,
				Description: "Описание проекта",
			},
		},
	}
}

func dataSourceProjectRead(ctx context.Context, d *schema.ResourceData, meta interface{}) diag.Diagnostics {
	config := meta.(*Config)

	projectID := d.Get("project_id").(int)

	req, err := http.NewRequestWithContext(ctx, "GET", config.buildURL(fmt.Sprintf("/projects/%d", projectID)), nil)
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
		return diag.Errorf("project with id %d not found", projectID)
	}

	if resp.StatusCode != http.StatusOK {
		return diag.Errorf("failed to read project: status %d", resp.StatusCode)
	}

	var project Project
	if err := json.NewDecoder(resp.Body).Decode(&project); err != nil {
		return diag.FromErr(err)
	}

	d.SetId(strconv.Itoa(project.ID))
	d.Set("name", project.Name)
	d.Set("description", project.Description)

	return nil
}
