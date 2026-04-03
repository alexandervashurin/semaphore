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

// Inventory представляет инвентарь Velum
type Inventory struct {
	ID        int    `json:"id,omitempty"`
	ProjectID int    `json:"project_id"`
	Name      string `json:"name"`
	Type      string `json:"type"` // static, static-yaml, file
	Inventory string `json:"inventory"`
}

func dataSourceInventory() *schema.Resource {
	return &schema.Resource{
		ReadContext: dataSourceInventoryRead,
		Schema: map[string]*schema.Schema{
			"project_id": {
				Type:        schema.TypeInt,
				Required:    true,
				Description: "ID проекта",
			},
			"inventory_id": {
				Type:        schema.TypeInt,
				Required:    true,
				Description: "ID инвентаря",
			},
			"name": {
				Type:        schema.TypeString,
				Computed:    true,
				Description: "Имя инвентаря",
			},
			"type": {
				Type:        schema.TypeString,
				Computed:    true,
				Description: "Тип инвентаря",
			},
		},
	}
}

func dataSourceInventoryRead(ctx context.Context, d *schema.ResourceData, meta interface{}) diag.Diagnostics {
	config := meta.(*Config)

	projectID := d.Get("project_id").(int)
	inventoryID := d.Get("inventory_id").(int)

	req, err := http.NewRequestWithContext(ctx, "GET", config.buildURL(fmt.Sprintf("/project/%d/inventory/%d", projectID, inventoryID)), nil)
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
		return diag.Errorf("inventory with id %d not found in project %d", inventoryID, projectID)
	}

	if resp.StatusCode != http.StatusOK {
		return diag.Errorf("failed to read inventory: status %d", resp.StatusCode)
	}

	var inventory Inventory
	if err := json.NewDecoder(resp.Body).Decode(&inventory); err != nil {
		return diag.FromErr(err)
	}

	d.SetId(strconv.Itoa(inventory.ID))
	d.Set("name", inventory.Name)
	d.Set("type", inventory.Type)

	return nil
}
