package provider

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"

	"github.com/hashicorp/terraform-plugin-framework/datasource"
	"github.com/hashicorp/terraform-plugin-framework/datasource/schema"
	"github.com/hashicorp/terraform-plugin-framework/types"
)

// TemplateDataSource defines the template data source.
type TemplateDataSource struct {
	client *VelumClient
}

// TemplateDataSourceModel describes the data source model.
type TemplateDataSourceModel struct {
	ID       types.Int64  `tfsdk:"id"`
	ProjectID types.Int64 `tfsdk:"project_id"`
	Name     types.String `tfsdk:"name"`
	Playbook types.String `tfsdk:"playbook"`
	Type     types.String `tfsdk:"type"`
}

func NewTemplateDataSource() datasource.DataSource {
	return &TemplateDataSource{}
}

func (d *TemplateDataSource) Metadata(ctx context.Context, req datasource.MetadataRequest, resp *datasource.MetadataResponse) {
	resp.TypeName = req.ProviderTypeName + "_template"
}

func (d *TemplateDataSource) Schema(ctx context.Context, req datasource.SchemaRequest, resp *datasource.SchemaResponse) {
	resp.Schema = schema.Schema{
		MarkdownDescription: "Look up a Velum template by name",
		Attributes: map[string]schema.Attribute{
			"id": schema.Int64Attribute{
				MarkdownDescription: "Template ID",
				Computed:            true,
			},
			"project_id": schema.Int64Attribute{
				MarkdownDescription: "Project ID",
				Required:            true,
			},
			"name": schema.StringAttribute{
				MarkdownDescription: "Template name",
				Required:            true,
			},
			"playbook": schema.StringAttribute{
				MarkdownDescription: "Playbook path",
				Computed:            true,
			},
			"type": schema.StringAttribute{
				MarkdownDescription: "Template type",
				Computed:            true,
			},
		},
	}
}

func (d *TemplateDataSource) Configure(ctx context.Context, req datasource.ConfigureRequest, resp *datasource.ConfigureResponse) {
	if req.ProviderData == nil {
		return
	}
	client, ok := req.ProviderData.(*VelumClient)
	if !ok {
		resp.Diagnostics.AddError("Unexpected Data Source Configure Type", "")
		return
	}
	d.client = client
}

func (d *TemplateDataSource) Read(ctx context.Context, req datasource.ReadRequest, resp *datasource.ReadResponse) {
	var config TemplateDataSourceModel
	resp.Diagnostics.Append(req.Config.Get(ctx, &config)...)
	if resp.Diagnostics.HasError() {
		return
	}

	httpResp, err := d.client.doRequest("GET", fmt.Sprintf("/api/project/%d/templates", config.ProjectID.ValueInt64()), nil)
	if err != nil {
		resp.Diagnostics.AddError("Failed to list templates", err.Error())
		return
	}
	defer httpResp.Body.Close()

	var templates []map[string]interface{}
	json.NewDecoder(httpResp.Body).Decode(&templates)

	for _, t := range templates {
		if t["name"] == config.Name.ValueString() {
			config.ID = types.Int64Value(int64(t["id"].(float64)))
			config.Playbook = types.StringValue(t["playbook"].(string))
			if v, ok := t["type"]; ok && v != nil {
				config.Type = types.StringValue(v.(string))
			}
			resp.Diagnostics.Append(resp.State.Set(ctx, &config)...)
			return
		}
	}

	resp.Diagnostics.AddError("Template not found", fmt.Sprintf("No template with name '%s'", config.Name.ValueString()))
}
