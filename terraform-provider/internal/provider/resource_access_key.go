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

// AccessKeyResource defines the access key resource.
type AccessKeyResource struct {
	client *VelumClient
}

// AccessKeyResourceModel describes the access key resource data model.
type AccessKeyResourceModel struct {
	ID          types.Int64  `tfsdk:"id"`
	ProjectID   types.Int64  `tfsdk:"project_id"`
	Name        types.String `tfsdk:"name"`
	Type        types.String `tfsdk:"type"`
	Secret      types.String `tfsdk:"secret"`
	SecretPhase types.String `tfsdk:"secret_phase"`
	LoginPass   types.String `tfsdk:"login_password"`
}

func NewAccessKeyResource() resource.Resource {
	return &AccessKeyResource{}
}

func (r *AccessKeyResource) Metadata(ctx context.Context, req resource.MetadataRequest, resp *resource.MetadataResponse) {
	resp.TypeName = req.ProviderTypeName + "_access_key"
}

func (r *AccessKeyResource) Schema(ctx context.Context, req resource.SchemaRequest, resp *resource.SchemaResponse) {
	resp.Schema = schema.Schema{
		MarkdownDescription: "Velum access key resource",
		Attributes: map[string]schema.Attribute{
			"id": schema.Int64Attribute{
				Computed: true,
			},
			"project_id": schema.Int64Attribute{
				MarkdownDescription: "Project ID",
				Required:            true,
			},
			"name": schema.StringAttribute{
				MarkdownDescription: "Access key name",
				Required:            true,
			},
			"type": schema.StringAttribute{
				MarkdownDescription: "Key type (none, ssh, login_password)",
				Optional:            true,
			},
			"secret": schema.StringAttribute{
				MarkdownDescription: "SSH private key content",
				Optional:            true,
				Sensitive:           true,
			},
			"secret_phase": schema.StringAttribute{
				MarkdownDescription: "SSH key passphrase",
				Optional:            true,
				Sensitive:           true,
			},
			"login_password": schema.StringAttribute{
				MarkdownDescription: "Login:password pair (JSON)",
				Optional:            true,
				Sensitive:           true,
			},
		},
	}
}

func (r *AccessKeyResource) Configure(ctx context.Context, req resource.ConfigureRequest, resp *resource.ConfigureResponse) {
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

func (r *AccessKeyResource) Create(ctx context.Context, req resource.CreateRequest, resp *resource.CreateResponse) {
	var plan AccessKeyResourceModel
	resp.Diagnostics.Append(req.Plan.Get(ctx, &plan)...)
	if resp.Diagnostics.HasError() {
		return
	}

	payload := map[string]interface{}{
		"name":       plan.Name.ValueString(),
		"project_id": plan.ProjectID.ValueInt64(),
	}
	if !plan.Type.IsNull() {
		payload["type"] = plan.Type.ValueString()
	}
	if !plan.Secret.IsNull() {
		payload["secret"] = plan.Secret.ValueString()
	}
	if !plan.SecretPhase.IsNull() {
		payload["secret_phrase"] = plan.SecretPhase.ValueString()
	}
	if !plan.LoginPass.IsNull() {
		payload["login_password"] = plan.LoginPass.ValueString()
	}

	body, _ := json.Marshal(payload)
	httpResp, err := r.client.doRequest("POST", "/api/access_keys", body)
	if err != nil {
		resp.Diagnostics.AddError("Failed to create access key", err.Error())
		return
	}
	defer httpResp.Body.Close()

	if httpResp.StatusCode != http.StatusCreated && httpResp.StatusCode != http.StatusOK {
		resp.Diagnostics.AddError("Failed to create access key", fmt.Sprintf("HTTP %d", httpResp.StatusCode))
		return
	}

	var result map[string]interface{}
	json.NewDecoder(httpResp.Body).Decode(&result)

	plan.ID = types.Int64Value(int64(result["id"].(float64)))
	resp.Diagnostics.Append(resp.State.Set(ctx, &plan)...)
}

func (r *AccessKeyResource) Read(ctx context.Context, req resource.ReadRequest, resp *resource.ReadResponse) {
	var state AccessKeyResourceModel
	resp.Diagnostics.Append(req.State.Get(ctx, &state)...)
	if resp.Diagnostics.HasError() {
		return
	}

	httpResp, err := r.client.doRequest("GET", fmt.Sprintf("/api/access_keys/%d", state.ID.ValueInt64()), nil)
	if err != nil {
		resp.Diagnostics.AddError("Failed to read access key", err.Error())
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

func (r *AccessKeyResource) Update(ctx context.Context, req resource.UpdateRequest, resp *resource.UpdateResponse) {
	var plan AccessKeyResourceModel
	resp.Diagnostics.Append(req.Plan.Get(ctx, &plan)...)
	if resp.Diagnostics.HasError() {
		return
	}

	payload := map[string]interface{}{
		"name": plan.Name.ValueString(),
	}
	body, _ := json.Marshal(payload)
	httpResp, err := r.client.doRequest("PUT", fmt.Sprintf("/api/access_keys/%d", plan.ID.ValueInt64()), body)
	if err != nil {
		resp.Diagnostics.AddError("Failed to update access key", err.Error())
		return
	}
	defer httpResp.Body.Close()

	if httpResp.StatusCode != http.StatusOK {
		resp.Diagnostics.AddError("Failed to update access key", fmt.Sprintf("HTTP %d", httpResp.StatusCode))
		return
	}

	resp.Diagnostics.Append(resp.State.Set(ctx, &plan)...)
}

func (r *AccessKeyResource) Delete(ctx context.Context, req resource.DeleteRequest, resp *resource.DeleteResponse) {
	var state AccessKeyResourceModel
	resp.Diagnostics.Append(req.State.Get(ctx, &state)...)
	if resp.Diagnostics.HasError() {
		return
	}

	httpResp, err := r.client.doRequest("DELETE", fmt.Sprintf("/api/access_keys/%d", state.ID.ValueInt64()), nil)
	if err != nil {
		resp.Diagnostics.AddError("Failed to delete access key", err.Error())
		return
	}
	defer httpResp.Body.Close()

	if httpResp.StatusCode != http.StatusNoContent && httpResp.StatusCode != http.StatusOK {
		resp.Diagnostics.AddError("Failed to delete access key", fmt.Sprintf("HTTP %d", httpResp.StatusCode))
	}
}

func (r *AccessKeyResource) ImportState(ctx context.Context, req resource.ImportStateRequest, resp *resource.ImportStateResponse) {
	resource.ImportStatePassthroughID(ctx, "id", req, resp)
}
