package main

import (
	"context"
	"flag"
	"log"

	"github.com/hashicorp/terraform-plugin-framework/providerserver"
	"terraform-provider-velum/internal/provider"
)

//go:generate terraform fmt -recursive ./examples/
//go:generate go run github.com/hashicorp/terraform-plugin-docs/cmd/tfplugindocs

func main() {
	var debug bool

	flag.BoolVar(&debug, "debug", false, "set to true to run the provider with supported debug methods")
	flag.Parse()

	opts := providerserver.ServeOpts{
		Address: "registry.terraform.io/alexandervashurin/velum",
		Debug:   debug,
	}

	err := providerserver.Serve(context.Background(), provider.New, opts)
	if err != nil {
		log.Fatalf("Error serving provider: %s", err.Error())
	}
}
