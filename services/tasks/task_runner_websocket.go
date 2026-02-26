package tasks

import (
	"encoding/json"
	"github.com/semaphoreui/semaphore/api/sockets"
	"github.com/semaphoreui/semaphore/util"
)

// sendWebSocketUpdate отправляет обновление статуса через WebSocket
func (t *TaskRunner) sendWebSocketUpdate() {
	for _, user := range t.users {
		b, err := json.Marshal(&map[string]any{
			"type":        "update",
			"status":      t.Task.Status,
			"task_id":     t.Task.ID,
			"template_id": t.Task.TemplateID,
			"project_id":  t.Task.ProjectID,
		})

		if err != nil {
			util.LogPanic(err)
			continue
		}

		sockets.Message(user, b)
	}
}

// notifyStatusChange уведомляет об изменении статуса
func (t *TaskRunner) notifyStatusChange(status task_logger.TaskStatus) {
	t.sendWebSocketUpdate()
	
	for _, l := range t.statusListeners {
		l(status)
	}
}

// notifyLog уведомляет о новом логе
func (t *TaskRunner) notifyLog(time time.Time, msg string) {
	for _, l := range t.logListeners {
		l(time, msg)
	}
}
