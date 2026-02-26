package tasks

import (
	"encoding/json"
	"fmt"

	"github.com/semaphoreui/semaphore/db"
)

// getCLIArgs возвращает аргументы CLI из шаблона и задачи
func (t *LocalJob) getCLIArgs() (templateArgs []string, taskArgs []string, err error) {
	templateArgs = []string{}
	taskArgs = []string{}

	if t.Template.Arguments != nil {
		err = json.Unmarshal([]byte(*t.Template.Arguments), &templateArgs)
		if err != nil {
			return
		}
	}

	if t.Task.Arguments != nil {
		err = json.Unmarshal([]byte(*t.Task.Arguments), &taskArgs)
		if err != nil {
			return
		}
	}

	return
}

// getCLIArgsMap возвращает аргументы CLI в виде карты (для Terraform)
func (t *LocalJob) getCLIArgsMap() (templateArgsMap map[string][]string, taskArgsMap map[string][]string, err error) {
	templateArgsMap = make(map[string][]string)
	taskArgsMap = make(map[string][]string)

	if t.Template.Arguments != nil {
		err = json.Unmarshal([]byte(*t.Template.Arguments), &templateArgsMap)
		if err != nil {
			// Если не удалось распарсить как map, пробуем как []string
			var templateArgs []string
			err = json.Unmarshal([]byte(*t.Template.Arguments), &templateArgs)
			if err == nil {
				templateArgsMap["default"] = templateArgs
			} else {
				return
			}
		}
	}

	if t.Task.Arguments != nil {
		err = json.Unmarshal([]byte(*t.Task.Arguments), &taskArgsMap)
		if err != nil {
			// Если не удалось распарсить как map, пробуем как []string
			var taskArgs []string
			err = json.Unmarshal([]byte(*t.Task.Arguments), &taskArgs)
			if err == nil {
				taskArgsMap["default"] = taskArgs
			} else {
				return
			}
		}
	}

	return
}

// getTemplateParams возвращает параметры шаблона
func (t *LocalJob) getTemplateParams() (any, error) {
	var params any
	err := json.Unmarshal([]byte(t.Template.Params), &params)
	if err != nil {
		return nil, err
	}
	return params, nil
}

// getParams возвращает параметры задачи
func (t *LocalJob) getParams() (params any, err error) {
	err = t.Task.ExtractParams(&params)
	return
}
