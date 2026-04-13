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

// ProjectDataSource defines the project data source.
type ProjectDataSource struct {
	client *VelumClient
}

// ProjectDataSourceModel describes the data source model.
type ProjectDataSourceModel struct {
	ID   types.Int64  `tfsdk:"id"`
	Name types.String `tfsdk:"name"`
}

func NewProjectDataSource() datasource.DataSource {
	return &ProjectDataSource{}
}

func (d *ProjectDataSource) Metadata(ctx context.Context, req datasource.MetadataRequest, resp *datasource.MetadataResponse) {
	resp.TypeName = req.ProviderTypeName + "_project"
}

func (d *ProjectDataSource) Schema(ctx context.Context, req datasource.SchemaRequest, resp *datasource.SchemaResponse) {
	resp.Schema = schema.Schema{
		MarkdownDescription: "Look up a Velum project by name",
		Attributes: map[string]schema.Attribute{
			"id": schema.Int64Attribute{
				MarkdownDescription: "Project ID",
				Computed:            true,
			},
			"name": schema.StringAttribute{
				MarkdownDescription: "Project name",
				Required:            true,
			},
		},
	}
}

func (d *ProjectDataSource) Configure(ctx context.Context, req datasource.ConfigureRequest, resp *datasource.ConfigureResponse) {
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

func (d *ProjectDataSource) Read(ctx context.Context, req datasource.ReadRequest, resp *datasource.ReadResponse) {
	var config ProjectDataSourceModel
	resp.Diagnostics.Append(req.Config.Get(ctx, &config)...)
	if resp.Diagnostics.HasError() {
		return
	}

	// Fetch all projects and find by name
	httpResp, err := d.client.doRequest("GET", "/api/projects", nil)
	if err != nil {
		resp.Diagnostics.AddError("Failed to list projects", err.Error())
		return
	}
	defer httpResp.Body.Close()

	var projects []map[string]interface{}
	json.NewDecoder(httpResp.Body).Decode(&projects)

	for _, p := range projects {
		if p["name"] == config.Name.ValueString() {
			config.ID = types.Int64Value(int64(p["id"].(float64)))
			resp.Diagnostics.Append(resp.State.Set(ctx, &config)...)
			return
		}
	}

	resp.Diagnostics.AddError("Project not found", fmt.Sprintf("No project with name '%s'", config.Name.ValueString()))
}
