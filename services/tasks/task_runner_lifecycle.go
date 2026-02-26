package tasks

import (
	"github.com/semaphoreui/semaphore/db"
	log "github.com/sirupsen/logrus"
)

// run запускает задачу
func (t *TaskRunner) run() {
	// Основная логика запуска задачи
	// Перемещено из TaskRunner.go
	
	t.Log("Task started")
	
	// Подготовка деталей
	if err := t.populateDetails(); err != nil {
		t.Log("Failed to populate details: " + err.Error())
		return
	}
	
	// Подготовка окружения
	if err := t.populateTaskEnvironment(); err != nil {
		t.Log("Failed to populate environment: " + err.Error())
		return
	}
	
	// Создание job
	t.job = &LocalJob{
		Task:         t.Task,
		Template:     t.Template,
		Inventory:    t.Inventory,
		Repository:   t.Repository,
		Environment:  t.Environment,
		Logger:       t,
		KeyInstaller: t.keyInstaller,
	}
	
	// Запуск задачи
	err := t.job.Run(t.Username, t.IncomingVersion, t.Alias)
	
	if err != nil {
		t.Log("Task failed: " + err.Error())
	} else {
		t.Log("Task completed successfully")
	}
	
	// Создание события задачи
	t.createTaskEvent()
}

// kill останавливает задачу
func (t *TaskRunner) kill() {
	if t.job != nil {
		t.job.Kill()
	}
}

// createTaskEvent создаёт событие задачи в БД
func (t *TaskRunner) createTaskEvent() {
	objType := db.EventTask
	desc := "Task " + strconv.Itoa(t.Task.ID) + " (" + t.Template.Name + ")" + " finished - " + strings.ToUpper(string(t.Task.Status))
	
	_, err := helpers.Store(t.pool.r.Context()).CreateEvent(db.Event{
		ObjectType:  objType,
		ObjectID:    t.Task.ID,
		ProjectID:   t.Task.ProjectID,
		Description: desc,
	})
	
	if err != nil {
		log.WithError(err).Error("Failed to create task event")
	}
}
