package tasks

import (
	"encoding/json"
	"github.com/semaphoreui/semaphore/api/sockets"
	"github.com/semaphoreui/semaphore/util"
	log "github.com/sirupsen/logrus"
)

// saveStatus сохраняет статус задачи и уведомляет пользователей
func (t *TaskRunner) saveStatus() {
	for _, user := range t.users {
		b, err := json.Marshal(&map[string]any{
			"type":        "update",
			"start":       t.Task.Start,
			"end":         t.Task.End,
			"status":      t.Task.Status,
			"task_id":     t.Task.ID,
			"template_id": t.Task.TemplateID,
			"project_id":  t.Task.ProjectID,
			"version":     t.Task.Version,
		})

		util.LogPanic(err)

		sockets.Message(user, b)
	}
	
	// Уведомление слушателей статусов
	for _, l := range t.statusListeners {
		l(t.Task.Status)
	}
}

// Log записывает лог задачи
func (t *TaskRunner) Log(msg string) {
	// Запись в БД
	_, err := t.pool.store.CreateTaskOutput(db.TaskOutput{
		TaskID: t.Task.ID,
		Output: msg,
		Time:   time.Now().UTC(),
	})
	
	if err != nil {
		log.WithError(err).Error("Failed to create task output")
	}
	
	// Уведомление слушателей логов
	now := time.Now().UTC()
	for _, l := range t.logListeners {
		l(now, msg)
	}
}

// SetStatus устанавливает статус задачи
func (t *TaskRunner) SetStatus(status task_logger.TaskStatus) {
	t.Task.Status = status
	t.saveStatus()
}
