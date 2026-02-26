package tasks

import (
	"fmt"

	"github.com/semaphoreui/semaphore/db"
)

// getShellArgs возвращает аргументы для shell скрипта
func (t *LocalJob) getShellArgs(username string, incomingVersion *string) (args []string, err error) {
	extraVars, err := t.getEnvironmentExtraVars(username, incomingVersion)

	if err != nil {
		t.Log(err.Error())
		t.Log("Error getting environment extra vars")
		return
	}

	templateArgs, taskArgs, err := t.getCLIArgs()
	if err != nil {
		t.Log(err.Error())
		return
	}

	// Script to run
	args = append(args, t.Template.Playbook)

	// Include Environment Secret Vars
	for _, secret := range t.Environment.Secrets {
		if secret.Type == db.EnvironmentSecretVar {
			args = append(args, fmt.Sprintf("%s=%s", secret.Name, secret.Secret))
		}
	}

	// Include extra args from template
	args = append(args, templateArgs...)

	// Include ExtraVars and Survey Vars
	for name, value := range extraVars {
		if name != "semaphore_vars" {
			args = append(args, fmt.Sprintf("%s=%s", name, value))
		}
	}

	// Include extra args from task
	args = append(args, taskArgs...)

	return
}

// getTerraformArgs возвращает аргументы для Terraform
func (t *LocalJob) getTerraformArgs(username string, incomingVersion *string) (argsMap map[string][]string, err error) {

	argsMap = make(map[string][]string)

	extraVars, err := t.getEnvironmentExtraVars(username, incomingVersion)

	if err != nil {
		t.Log(err.Error())
		t.Log("Could not remove command environment, if existent it will be passed to --extra-vars. This is not fatal but be aware of side effects")
		return
	}

	var params db.TerraformTaskParams
	err = t.Task.ExtractParams(&params)
	if err != nil {
		return
	}

	// Common args for destroy flag
	destroyArgs := []string{}
	if params.Destroy {
		destroyArgs = append(destroyArgs, "-destroy")
	}

	// Common args for environment variables
	varArgs := []string{}
	for name, value := range extraVars {
		if name == "semaphore_vars" {
			continue
		}
		varArgs = append(varArgs, "-var", fmt.Sprintf("%s=%s", name, value))
	}

	templateArgsMap, taskArgsMap, err := t.getCLIArgsMap()
	if err != nil {
		t.Log(err.Error())
		return
	}

	// Common args for environment secrets
	secretArgs := []string{}
	for _, secret := range t.Environment.Secrets {
		if secret.Type != db.EnvironmentSecretVar {
			continue
		}
		secretArgs = append(secretArgs, "-var", fmt.Sprintf("%s=%s", secret.Name, secret.Secret))
	}

	// Merge template and task args maps
	if templateArgsMap != nil {
		for stage, stageArgs := range templateArgsMap {
			argsMap[stage] = append([]string{}, stageArgs...)
		}
	}

	if taskArgsMap != nil {
		for stage, stageArgs := range taskArgsMap {
			if existing, ok := argsMap[stage]; ok {
				argsMap[stage] = append(existing, stageArgs...)
			} else {
				argsMap[stage] = append([]string{}, stageArgs...)
			}
		}
	}

	if len(argsMap) == 0 {
		argsMap["default"] = []string{}
	}

	// Add common args to each stage except init
	for stage := range argsMap {
		if stage == "init" {
			continue
		}
		// Prepend destroy args
		combined := append([]string{}, destroyArgs...)
		combined = append(combined, argsMap[stage]...)
		combined = append(combined, varArgs...)
		combined = append(combined, secretArgs...)
		argsMap[stage] = combined
	}

	return
}
