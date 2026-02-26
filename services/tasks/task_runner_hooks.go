package tasks

import (
	"github.com/semaphoreui/semaphore/services/tasks/hooks"
	"github.com/semaphoreui/semaphore/pro_interfaces"
)

// runHooks запускает hooks для задачи
func (t *TaskRunner) runHooks(eventType string) error {
	// Получение hooks из шаблона
	hooksList := t.Template.Hooks
	
	if hooksList == nil {
		return nil
	}
	
	// Запуск hooks для указанного события
	for _, hook := range hooksList {
		if hook.Event == eventType {
			err := hooks.ExecuteHook(hook, t)
			if err != nil {
				t.Log("Hook failed: " + err.Error())
				// Продолжаем выполнение даже если hook не удался
			}
		}
	}
	
	return nil
}

// getHookExecutor возвращает executor для hooks
func (t *TaskRunner) getHookExecutor() pro_interfaces.HookExecutor {
	// TODO: Интеграция с PRO hook executor
	return nil
}
