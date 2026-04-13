package provider

import (
	"context"
	"fmt"
	"net/http"

	"github.com/hashicorp/terraform-plugin-framework/datasource"
	"github.com/hashicorp/terraform-plugin-framework/provider"
	"github.com/hashicorp/terraform-plugin-framework/provider/schema"
	"github.com/hashicorp/terraform-plugin-framework/resource"
	"github.com/hashicorp/terraform-plugin-framework/types"
)

// VelumProvider defines the provider implementation.
type VelumProvider struct {
	version string
}

// VelumProviderModel describes the provider data model.
type VelumProviderModel struct {
	ServerURL types.String `tfsdk:"server_url"`
	APIToken  types.String `tfsdk:"api_token"`
}

// VelumClient is the HTTP client for Velum API.
type VelumClient struct {
	BaseURL  string
	APIToken string
	Client   *http.Client
}

func (c *VelumClient) doRequest(method, path string, body []byte) (*http.Response, error) {
	req, err := http.NewRequest(method, fmt.Sprintf("%s%s", c.BaseURL, path), nil)
	if err != nil {
		return nil, err
	}
	req.Header.Set("Authorization", fmt.Sprintf("Bearer %s", c.APIToken))
	req.Header.Set("Content-Type", "application/json")
	return c.Client.Do(req)
}

func (p *VelumProvider) Metadata(ctx context.Context, req provider.MetadataRequest, resp *provider.MetadataResponse) {
	resp.TypeName = "velum"
	resp.Version = p.version
}

func (p *VelumProvider) Schema(ctx context.Context, req provider.SchemaRequest, resp *provider.SchemaResponse) {
	resp.Schema = schema.Schema{
		Attributes: map[string]schema.Attribute{
			"server_url": schema.StringAttribute{
				MarkdownDescription: "Velum server URL (e.g. http://localhost:3000)",
				Optional:            true,
			},
			"api_token": schema.StringAttribute{
				MarkdownDescription: "Velum API token",
				Optional:            true,
				Sensitive:           true,
			},
		},
	}
}

func (p *VelumProvider) Configure(ctx context.Context, req provider.ConfigureRequest, resp *provider.ConfigureResponse) {
	var data VelumProviderModel

	resp.Diagnostics.Append(req.Config.Get(ctx, &data)...)
	if resp.Diagnostics.HasError() {
		return
	}

	client := &VelumClient{
		BaseURL:  data.ServerURL.ValueString(),
		APIToken: data.APIToken.ValueString(),
		Client:   &http.Client{},
	}

	resp.DataSourceData = client
	resp.ResourceData = client
}

func (p *VelumProvider) Resources(ctx context.Context) []func() resource.Resource {
	return []func() resource.Resource{
		NewProjectResource,
		NewTemplateResource,
		NewAccessKeyResource,
	}
}

func (p *VelumProvider) DataSources(ctx context.Context) []func() datasource.DataSource {
	return []func() datasource.DataSource{
		NewProjectDataSource,
		NewTemplateDataSource,
	}
}

func New(version string) func() provider.Provider {
	return func() provider.Provider {
		return &VelumProvider{
			version: version,
		}
	}
}
