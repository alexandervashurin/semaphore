package main

import (
	"flag"
	"log"

	"github.com/hashicorp/terraform-plugin-sdk/v2/plugin"
	"terraform-provider-velum/velum"
)

func main() {
	var debugMode bool

	flag.BoolVar(&debugMode, "debug", false, "set to true to run the provider with support for debuggers like delve")
	flag.Parse()

	opts := &plugin.ServeOpts{
		Debug:        debugMode,
		ProviderAddr: "registry.terraform.io/tnl-o/velum",
		ProviderFunc: velum.Provider,
	}

	plugin.Serve(opts)

	log.Println("Terraform Provider Velum started")
}
