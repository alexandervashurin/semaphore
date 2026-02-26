package tasks

import (
	"errors"
	"strings"
	log "github.com/sirupsen/logrus"
)

// prepareError подготавливает ошибку для логирования
func (t *TaskRunner) prepareError(err error, errMsg string) error {
	if err == nil {
		return nil
	}
	
	// Логирование ошибки
	log.WithError(err).Error(errMsg)
	
	// Добавление контекста
	wrappedErr := errors.New(errMsg + ": " + err.Error())
	
	return wrappedErr
}

// isErrorFatal проверяет, является ли ошибка фатальной
func (t *TaskRunner) isErrorFatal(err error) bool {
	if err == nil {
		return false
	}
	
	// Проверка на фатальные ошибки
	fatalErrors := []string{
		"permission denied",
		"authentication failed",
		"connection refused",
	}
	
	errStr := strings.ToLower(err.Error())
	
	for _, fatal := range fatalErrors {
		if strings.Contains(errStr, fatal) {
			return true
		}
	}
	
	return false
}

// logError логирует ошибку с контекстом
func (t *TaskRunner) logError(err error, context string) {
	if err != nil {
		t.Log(context + ": " + err.Error())
		log.WithField("task_id", t.Task.ID).WithError(err).Error(context)
	}
}
