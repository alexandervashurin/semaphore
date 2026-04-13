package provider

import (
	"context"
	"encoding/json"
	"fmt"
	"net/http"

	"github.com/hashicorp/terraform-plugin-framework/resource"
	"github.com/hashicorp/terraform-plugin-framework/resource/schema"
	"github.com/hashicorp/terraform-plugin-framework/types"
)

// TemplateResource defines the template resource.
type TemplateResource struct {
	client *VelumClient
}

// TemplateResourceModel describes the template resource data model.
type TemplateResourceModel struct {
	ID            types.Int64  `tfsdk:"id"`
	ProjectID     types.Int64  `tfsdk:"project_id"`
	Name          types.String `tfsdk:"name"`
	Playbook      types.String `tfsdk:"playbook"`
	Description   types.String `tfsdk:"description"`
	Type          types.String `tfsdk:"type"`
	App           types.String `tfsdk:"app"`
	InventoryID   types.Int64  `tfsdk:"inventory_id"`
	RepositoryID  types.Int64  `tfsdk:"repository_id"`
	EnvironmentID types.Int64  `tfsdk:"environment_id"`
	Arguments     types.String `tfsdk:"arguments"`
	GitBranch     types.String `tfsdk:"git_branch"`
}

func NewTemplateResource() resource.Resource {
	return &TemplateResource{}
}

func (r *TemplateResource) Metadata(ctx context.Context, req resource.MetadataRequest, resp *resource.MetadataResponse) {
	resp.TypeName = req.ProviderTypeName + "_template"
}

func (r *TemplateResource) Schema(ctx context.Context, req resource.SchemaRequest, resp *resource.SchemaResponse) {
	resp.Schema = schema.Schema{
		MarkdownDescription: "Velum template resource",
		Attributes: map[string]schema.Attribute{
			"id": schema.Int64Attribute{
				Computed: true,
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
				MarkdownDescription: "Path to playbook file",
				Required:            true,
			},
			"description": schema.StringAttribute{
				MarkdownDescription: "Template description",
				Optional:            true,
			},
			"type": schema.StringAttribute{
				MarkdownDescription: "Template type (ansible, terraform, shell, etc.)",
				Optional:            true,
			},
			"app": schema.StringAttribute{
				MarkdownDescription: "Application type",
				Optional:            true,
			},
			"inventory_id": schema.Int64Attribute{
				MarkdownDescription: "Inventory ID",
				Optional:            true,
			},
			"repository_id": schema.Int64Attribute{
				MarkdownDescription: "Repository ID",
				Optional:            true,
			},
			"environment_id": schema.Int64Attribute{
				MarkdownDescription: "Environment ID",
				Optional:            true,
			},
			"arguments": schema.StringAttribute{
				MarkdownDescription: "Task arguments (JSON)",
				Optional:            true,
			},
			"git_branch": schema.StringAttribute{
				MarkdownDescription: "Git branch",
				Optional:            true,
			},
		},
	}
}

func (r *TemplateResource) Configure(ctx context.Context, req resource.ConfigureRequest, resp *resource.ConfigureResponse) {
	if req.ProviderData == nil {
		return
	}
	client, ok := req.ProviderData.(*VelumClient)
	if !ok {
		resp.Diagnostics.AddError("Unexpected Resource Configure Type", "")
		return
	}
	r.client = client
}

func (r *TemplateResource) Create(ctx context.Context, req resource.CreateRequest, resp *resource.CreateResponse) {
	var plan TemplateResourceModel
	resp.Diagnostics.Append(req.Plan.Get(ctx, &plan)...)
	if resp.Diagnostics.HasError() {
		return
	}

	payload := map[string]interface{}{
		"name":     plan.Name.ValueString(),
		"playbook": plan.Playbook.ValueString(),
	}
	if !plan.ProjectID.IsNull() {
		payload["project_id"] = plan.ProjectID.ValueInt64()
	}
	if !plan.Type.IsNull() {
		payload["type"] = plan.Type.ValueString()
	}
	if !plan.InventoryID.IsNull() {
		payload["inventory_id"] = plan.InventoryID.ValueInt64()
	}
	if !plan.RepositoryID.IsNull() {
		payload["repository_id"] = plan.RepositoryID.ValueInt64()
	}
	if !plan.EnvironmentID.IsNull() {
		payload["environment_id"] = plan.EnvironmentID.ValueInt64()
	}

	body, _ := json.Marshal(payload)
	httpResp, err := r.client.doRequest("POST", "/api/templates", body)
	if err != nil {
		resp.Diagnostics.AddError("Failed to create template", err.Error())
		return
	}
	defer httpResp.Body.Close()

	if httpResp.StatusCode != http.StatusCreated && httpResp.StatusCode != http.StatusOK {
		resp.Diagnostics.AddError("Failed to create template", fmt.Sprintf("HTTP %d", httpResp.StatusCode))
		return
	}

	var result map[string]interface{}
	json.NewDecoder(httpResp.Body).Decode(&result)

	plan.ID = types.Int64Value(int64(result["id"].(float64)))
	resp.Diagnostics.Append(resp.State.Set(ctx, &plan)...)
}

func (r *TemplateResource) Read(ctx context.Context, req resource.ReadRequest, resp *resource.ReadResponse) {
	var state TemplateResourceModel
	resp.Diagnostics.Append(req.State.Get(ctx, &state)...)
	if resp.Diagnostics.HasError() {
		return
	}

	httpResp, err := r.client.doRequest("GET", fmt.Sprintf("/api/templates/%d", state.ID.ValueInt64()), nil)
	if err != nil {
		resp.Diagnostics.AddError("Failed to read template", err.Error())
		return
	}
	defer httpResp.Body.Close()

	if httpResp.StatusCode == http.StatusNotFound {
		resp.State.RemoveResource(ctx)
		return
	}

	var result map[string]interface{}
	json.NewDecoder(httpResp.Body).Decode(&result)

	state.Name = types.StringValue(result["name"].(string))
	state.Playbook = types.StringValue(result["playbook"].(string))
	resp.Diagnostics.Append(resp.State.Set(ctx, &state)...)
}

func (r *TemplateResource) Update(ctx context.Context, req resource.UpdateRequest, resp *resource.UpdateResponse) {
	var plan TemplateResourceModel
	resp.Diagnostics.Append(req.Plan.Get(ctx, &plan)...)
	if resp.Diagnostics.HasError() {
		return
	}

	payload := map[string]interface{}{
		"name":     plan.Name.ValueString(),
		"playbook": plan.Playbook.ValueString(),
	}
	body, _ := json.Marshal(payload)
	httpResp, err := r.client.doRequest("PUT", fmt.Sprintf("/api/templates/%d", plan.ID.ValueInt64()), body)
	if err != nil {
		resp.Diagnostics.AddError("Failed to update template", err.Error())
		return
	}
	defer httpResp.Body.Close()

	if httpResp.StatusCode != http.StatusOK {
		resp.Diagnostics.AddError("Failed to update template", fmt.Sprintf("HTTP %d", httpResp.StatusCode))
		return
	}

	resp.Diagnostics.Append(resp.State.Set(ctx, &plan)...)
}

func (r *TemplateResource) Delete(ctx context.Context, req resource.DeleteRequest, resp *resource.DeleteResponse) {
	var state TemplateResourceModel
	resp.Diagnostics.Append(req.State.Get(ctx, &state)...)
	if resp.Diagnostics.HasError() {
		return
	}

	httpResp, err := r.client.doRequest("DELETE", fmt.Sprintf("/api/templates/%d", state.ID.ValueInt64()), nil)
	if err != nil {
		resp.Diagnostics.AddError("Failed to delete template", err.Error())
		return
	}
	defer httpResp.Body.Close()

	if httpResp.StatusCode != http.StatusNoContent && httpResp.StatusCode != http.StatusOK {
		resp.Diagnostics.AddError("Failed to delete template", fmt.Sprintf("HTTP %d", httpResp.StatusCode))
	}
}

func (r *TemplateResource) ImportState(ctx context.Context, req resource.ImportStateRequest, resp *resource.ImportStateResponse) {
	resource.ImportStatePassthroughID(ctx, "id", req, resp)
}
