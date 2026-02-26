package tasks

import (
	"github.com/semaphoreui/semaphore/db"
)

// populateDetails загружает детали задачи из БД
func (t *TaskRunner) populateDetails() error {
	var err error
	
	// Загрузка шаблона
	t.Template, err = t.pool.store.GetTemplate(t.Task.TemplateID)
	if err != nil {
		return err
	}
	
	// Загрузка инвентаря
	if t.Task.InventoryID != nil {
		t.Inventory, err = t.pool.store.GetInventory(t.Template.ProjectID, *t.Task.InventoryID)
		if err != nil {
			return err
		}
	}
	
	// Загрузка репозитория
	if t.Task.RepositoryID != nil {
		t.Repository, err = t.pool.store.GetRepository(t.Template.ProjectID, *t.Task.RepositoryID)
		if err != nil {
			return err
		}
	}
	
	// Загрузка окружения
	if t.Task.EnvironmentID != nil {
		t.Environment, err = t.pool.store.GetEnvironment(t.Template.ProjectID, *t.Task.EnvironmentID)
		if err != nil {
			return err
		}
	}
	
	return nil
}

// populateTaskEnvironment подготавливает окружение задачи
func (t *TaskRunner) populateTaskEnvironment() (err error) {
	// Получение пользователей для уведомлений
	t.users, err = t.pool.store.GetTemplateUsers(t.Task.TemplateID)
	if err != nil {
		return err
	}
	
	// Получение алертов
	t.alert, t.alertChat, err = t.pool.store.GetTaskAlertChat(t.Task.TemplateID)
	if err != nil {
		return err
	}
	
	return nil
}
