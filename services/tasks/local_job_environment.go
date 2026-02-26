package tasks

import (
	"encoding/json"
	"fmt"
	"maps"
	"strconv"
	"strings"

	"github.com/semaphoreui/semaphore/db"
	"github.com/semaphoreui/semaphore/util"
)

// getTaskDetails возвращает детали задачи в виде карты
func (t *LocalJob) getTaskDetails(username string, incomingVersion *string) (taskDetails map[string]any) {
	taskDetails = make(map[string]any)

	taskDetails["id"] = t.Task.ID

	if t.Task.Message != "" {
		taskDetails["message"] = t.Task.Message
	}

	taskDetails["username"] = username
	taskDetails["url"] = t.Task.GetUrl()
	taskDetails["commit_hash"] = t.Task.CommitHash
	taskDetails["commit_message"] = t.Task.CommitMessage
	taskDetails["inventory_name"] = t.Inventory.Name
	taskDetails["inventory_id"] = t.Inventory.ID
	taskDetails["repository_name"] = t.Repository.Name
	taskDetails["repository_id"] = t.Repository.ID

	if t.Template.Type != db.TemplateTask {
		taskDetails["type"] = t.Template.Type
		if incomingVersion != nil {
			taskDetails["incoming_version"] = incomingVersion
		}
		if t.Template.Type == db.TemplateBuild {
			taskDetails["target_version"] = t.Task.Version
		}
	}

	return
}

// getEnvironmentExtraVars возвращает дополнительные переменные из окружения
func (t *LocalJob) getEnvironmentExtraVars(username string, incomingVersion *string) (extraVars map[string]any, err error) {
	extraVars = make(map[string]any)

	if t.Environment.JSON != "" {
		err = json.Unmarshal([]byte(t.Environment.JSON), &extraVars)
		if err != nil {
			return
		}
	}

	vars := make(map[string]any)
	vars["task_details"] = t.getTaskDetails(username, incomingVersion)
	extraVars["semaphore_vars"] = vars

	return
}

// getEnvironmentExtraVarsJSON возвращает JSON дополнительных переменных
func (t *LocalJob) getEnvironmentExtraVarsJSON(username string, incomingVersion *string) (str string, err error) {
	extraVars := make(map[string]any)
	extraSecretVars := make(map[string]any)

	if t.Environment.JSON != "" {
		err = json.Unmarshal([]byte(t.Environment.JSON), &extraVars)
		if err != nil {
			return
		}
	}
	if t.Secret != "" {
		err = json.Unmarshal([]byte(t.Secret), &extraSecretVars)
		if err != nil {
			return
		}
	}
	t.Secret = "{}"

	maps.Copy(extraVars, extraSecretVars)

	vars := make(map[string]any)
	vars["task_details"] = t.getTaskDetails(username, incomingVersion)
	extraVars["semaphore_vars"] = vars

	ev, err := json.Marshal(extraVars)
	if err != nil {
		return
	}

	str = string(ev)

	return
}

// getEnvironmentENV возвращает переменные окружения ENV
func (t *LocalJob) getEnvironmentENV() (res []string, err error) {
	environmentVars := make(map[string]string)

	if t.Environment.ENV != nil {
		err = json.Unmarshal([]byte(*t.Environment.ENV), &environmentVars)
		if err != nil {
			return
		}
	}

	for key, val := range environmentVars {
		res = append(res, fmt.Sprintf("%s=%s", key, val))
	}

	for _, secret := range t.Environment.Secrets {
		if secret.Type != db.EnvironmentSecretEnv {
			continue
		}
		res = append(res, fmt.Sprintf("%s=%s", secret.Name, secret.Secret))
	}

	return
}

// getShellEnvironmentExtraENV возвращает дополнительные shell переменные окружения
func (t *LocalJob) getShellEnvironmentExtraENV(username string, incomingVersion *string) (extraShellVars []string) {
	taskDetails := t.getTaskDetails(username, incomingVersion)

	for taskDetail, taskDetailValue := range taskDetails {
		envVarName := fmt.Sprintf("SEMAPHORE_TASK_DETAILS_%s", strings.ToUpper(taskDetail))

		detailAsStr := ""
		switch taskDetailValueOfType := taskDetailValue.(type) {
		case string:
			detailAsStr = taskDetailValueOfType
		case *string:
			if taskDetailValueOfType != nil {
				detailAsStr = *taskDetailValueOfType
			}

		case int:
			detailAsStr = strconv.Itoa(taskDetailValueOfType)
		case *int:
			if taskDetailValueOfType != nil {
				detailAsStr = strconv.Itoa(*taskDetailValueOfType)
			}

		default:
			continue
		}

		if detailAsStr != "" {
			extraShellVars = append(extraShellVars, fmt.Sprintf("%s=%s", envVarName, util.ShellQuote(util.ShellStripUnsafe(detailAsStr))))
		}
	}

	return
}
