package tasks

import (
	"github.com/semaphoreui/semaphore/db_lib"
	"github.com/semaphoreui/semaphore/db"
	"github.com/semaphoreui/semaphore/pkg/task_logger"
)

// Job interface определяет методы для выполнения задачи
type Job interface {
	Run(username string, incomingVersion *string, alias string) error
	Kill()
	IsKilled() bool
}

// TaskRunner представляет выполняющуюся задачу
type TaskRunner struct {
	Task        db.Task
	Template    db.Template
	Inventory   db.Inventory
	Repository  db.Repository
	Environment db.Environment

	currentStage  *db.TaskStage
	currentOutput *db.TaskOutput
	currentState  any

	users        []int
	alert        bool
	alertChat    *string
	pool         *TaskPool
	keyInstaller db_lib.AccessKeyInstaller

	// job executes Ansible and returns stdout to Semaphore logs
	job Job

	RunnerID        int
	Username        string
	IncomingVersion *string

	statusListeners []task_logger.StatusListener
	logListeners    []task_logger.LogListener

	// Alias uses if task require an alias for run.
	// For example, terraform task require an alias for run.
	Alias string

	logWG sync.WaitGroup
}

// NewTaskRunner создаёт новый TaskRunner
func NewTaskRunner(
	newTask db.Task,
	p *TaskPool,
	username string,
	keyInstaller db_lib.AccessKeyInstaller,
) *TaskRunner {
	return &TaskRunner{
		Task:         newTask,
		pool:         p,
		Username:     username,
		keyInstaller: keyInstaller,
	}
}

// AddStatusListener добавляет слушателя статусов
func (t *TaskRunner) AddStatusListener(l task_logger.StatusListener) {
	t.statusListeners = append(t.statusListeners, l)
}

// AddLogListener добавляет слушателя логов
func (t *TaskRunner) AddLogListener(l task_logger.LogListener) {
	t.logListeners = append(t.logListeners, l)
}
