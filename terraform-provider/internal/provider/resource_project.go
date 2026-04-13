package provider

import (
	"context"
	"encoding/json"
	"fmt"
	"io"
	"net/http"

	"github.com/hashicorp/terraform-plugin-framework/resource"
	"github.com/hashicorp/terraform-plugin-framework/resource/schema"
	"github.com/hashicorp/terraform-plugin-framework/types"
)

// ProjectResource defines the project resource.
type ProjectResource struct {
	client *VelumClient
}

// ProjectResourceModel describes the project resource data model.
type ProjectResourceModel struct {
	ID               types.Int64  `tfsdk:"id"`
	Name             types.String `tfsdk:"name"`
	Description      types.String `tfsdk:"description"`
	Alert            types.Bool   `tfsdk:"alert"`
	AlertChat        types.String `tfsdk:"alert_chat"`
	MaxParallelTasks types.Int64  `tfsdk:"max_parallel_tasks"`
	Type             types.String `tfsdk:"type"`
}

func NewProjectResource() resource.Resource {
	return &ProjectResource{}
}

func (r *ProjectResource) Metadata(ctx context.Context, req resource.MetadataRequest, resp *resource.MetadataResponse) {
	resp.TypeName = req.ProviderTypeName + "_project"
}

func (r *ProjectResource) Schema(ctx context.Context, req resource.SchemaRequest, resp *resource.SchemaResponse) {
	resp.Schema = schema.Schema{
		MarkdownDescription: "Velum project resource",
		Attributes: map[string]schema.Attribute{
			"id": schema.Int64Attribute{
				Computed: true,
			},
			"name": schema.StringAttribute{
				MarkdownDescription: "Project name",
				Required:            true,
			},
			"description": schema.StringAttribute{
				MarkdownDescription: "Project description",
				Optional:            true,
			},
			"alert": schema.BoolAttribute{
				MarkdownDescription: "Enable notifications",
				Optional:            true,
			},
			"alert_chat": schema.StringAttribute{
				MarkdownDescription: "Chat ID for notifications",
				Optional:            true,
			},
			"max_parallel_tasks": schema.Int64Attribute{
				MarkdownDescription: "Max parallel tasks",
				Optional:            true,
			},
			"type": schema.StringAttribute{
				MarkdownDescription: "Project type",
				Optional:            true,
			},
		},
	}
}

func (r *ProjectResource) Configure(ctx context.Context, req resource.ConfigureRequest, resp *resource.ConfigureResponse) {
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

func (r *ProjectResource) Create(ctx context.Context, req resource.CreateRequest, resp *resource.CreateResponse) {
	var plan ProjectResourceModel
	resp.Diagnostics.Append(req.Plan.Get(ctx, &plan)...)
	if resp.Diagnostics.HasError() {
		return
	}

	payload := map[string]interface{}{
		"name": plan.Name.ValueString(),
	}
	if !plan.Description.IsNull() {
		payload["description"] = plan.Description.ValueString()
	}

	body, _ := json.Marshal(payload)
	httpResp, err := r.client.doRequest("POST", "/api/projects", body)
	if err != nil {
		resp.Diagnostics.AddError("Failed to create project", err.Error())
		return
	}
	defer httpResp.Body.Close()

	if httpResp.StatusCode != http.StatusCreated && httpResp.StatusCode != http.StatusOK {
		resp.Diagnostics.AddError("Failed to create project", fmt.Sprintf("HTTP %d", httpResp.StatusCode))
		return
	}

	var result map[string]interface{}
	json.NewDecoder(httpResp.Body).Decode(&result)

	plan.ID = types.Int64Value(int64(result["id"].(float64)))
	resp.Diagnostics.Append(resp.State.Set(ctx, &plan)...)
}

func (r *ProjectResource) Read(ctx context.Context, req resource.ReadRequest, resp *resource.ReadResponse) {
	var state ProjectResourceModel
	resp.Diagnostics.Append(req.State.Get(ctx, &state)...)
	if resp.Diagnostics.HasError() {
		return
	}

	httpResp, err := r.client.doRequest("GET", fmt.Sprintf("/api/projects/%d", state.ID.ValueInt64()), nil)
	if err != nil {
		resp.Diagnostics.AddError("Failed to read project", err.Error())
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
	resp.Diagnostics.Append(resp.State.Set(ctx, &state)...)
}

func (r *ProjectResource) Update(ctx context.Context, req resource.UpdateRequest, resp *resource.UpdateResponse) {
	var plan ProjectResourceModel
	resp.Diagnostics.Append(req.Plan.Get(ctx, &plan)...)
	if resp.Diagnostics.HasError() {
		return
	}

	payload := map[string]interface{}{
		"name": plan.Name.ValueString(),
	}
	body, _ := json.Marshal(payload)
	httpResp, err := r.client.doRequest("PUT", fmt.Sprintf("/api/projects/%d", plan.ID.ValueInt64()), body)
	if err != nil {
		resp.Diagnostics.AddError("Failed to update project", err.Error())
		return
	}
	defer httpResp.Body.Close()

	if httpResp.StatusCode != http.StatusOK {
		resp.Diagnostics.AddError("Failed to update project", fmt.Sprintf("HTTP %d", httpResp.StatusCode))
		return
	}

	resp.Diagnostics.Append(resp.State.Set(ctx, &plan)...)
}

func (r *ProjectResource) Delete(ctx context.Context, req resource.DeleteRequest, resp *resource.DeleteResponse) {
	var state ProjectResourceModel
	resp.Diagnostics.Append(req.State.Get(ctx, &state)...)
	if resp.Diagnostics.HasError() {
		return
	}

	httpResp, err := r.client.doRequest("DELETE", fmt.Sprintf("/api/projects/%d", state.ID.ValueInt64()), nil)
	if err != nil {
		resp.Diagnostics.AddError("Failed to delete project", err.Error())
		return
	}
	defer httpResp.Body.Close()

	if httpResp.StatusCode != http.StatusNoContent && httpResp.StatusCode != http.StatusOK {
		resp.Diagnostics.AddError("Failed to delete project", fmt.Sprintf("HTTP %d", httpResp.StatusCode))
	}
}

func (r *ProjectResource) ImportState(ctx context.Context, req resource.ImportStateRequest, resp *resource.ImportStateResponse) {
	resource.ImportStatePassthroughID(ctx, "id", req, resp)
}

func (r *ProjectResource) readProject(ctx context.Context, id int64) (*ProjectResourceModel, error) {
	httpResp, err := r.client.doRequest("GET", fmt.Sprintf("/api/projects/%d", id), nil)
	if err != nil {
		return nil, err
	}
	defer httpResp.Body.Close()

	if httpResp.StatusCode == http.StatusNotFound {
		return nil, fmt.Errorf("project not found")
	}

	var result map[string]interface{}
	body, _ := io.ReadAll(httpResp.Body)
	json.Unmarshal(body, &result)

	model := &ProjectResourceModel{
		ID:   types.Int64Value(id),
		Name: types.StringValue(result["name"].(string)),
	}
	return model, nil
}
