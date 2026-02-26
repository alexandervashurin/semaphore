package tasks

import (
	"os"

	"github.com/semaphoreui/semaphore/db"
	"github.com/semaphoreui/semaphore/db_lib"
	"github.com/semaphoreui/semaphore/pkg/ssh"
	"github.com/semaphoreui/semaphore/pkg/task_logger"
)

// LocalJob представляет задачу для локального выполнения
type LocalJob struct {
	Task        db.Task
	Template    db.Template
	Inventory   db.Inventory
	Repository  db.Repository
	Environment db.Environment
	Secret      string             // Secret содержит секретные переменные из Survey
	Logger      task_logger.Logger // Logger позволяет отправлять логи и статусы на сервер

	App db_lib.LocalApp

	killed  bool // killed означает, что получен запрос на остановку задачи из API
	Process *os.Process

	sshKeyInstallation     ssh.AccessKeyInstallation
	becomeKeyInstallation  ssh.AccessKeyInstallation
	vaultFileInstallations map[string]ssh.AccessKeyInstallation

	KeyInstaller db_lib.AccessKeyInstaller
}

// IsKilled возвращает true, если задача была убита
func (t *LocalJob) IsKilled() bool {
	return t.killed
}

// Kill останавливает задачу
func (t *LocalJob) Kill() {
	t.killed = true

	if t.Process == nil {
		return
	}

	err := t.Process.Kill()
	if err != nil {
		t.Log(err.Error())
	}
}

// Log записывает сообщение в лог
func (t *LocalJob) Log(msg string) {
	t.Logger.Log(msg)
}

// SetStatus устанавливает статус задачи
func (t *LocalJob) SetStatus(status task_logger.TaskStatus) {
	t.Logger.SetStatus(status)
}

// SetCommit устанавливает информацию о коммите
func (t *LocalJob) SetCommit(hash, message string) {
	// TODO: is this the correct place to do?
	t.Task.CommitHash = &hash
	t.Task.CommitMessage = message
	t.Logger.SetCommit(hash, message)
}
