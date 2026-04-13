import * as vscode from 'vscode';

export interface VelumProject {
    id: number;
    name: string;
    created: string;
    alert?: boolean;
    alert_chat?: string;
    max_parallel_tasks?: number;
}

export interface VelumTemplate {
    id: number;
    project_id: number;
    name: string;
    playbook: string;
    description?: string;
    type: string;
    app?: string;
    inventory_id?: number;
    repository_id?: number;
    environment_id?: number;
    arguments?: string;
}

export interface VelumTask {
    id: number;
    template_id: number;
    project_id: number;
    status: string;
    created: string;
    start?: string;
    end?: string;
    user_id?: number;
}

export class VelumApi {
    private serverUrl: string;
    private apiToken: string;

    constructor() {
        const config = vscode.workspace.getConfiguration('velum');
        this.serverUrl = config.get('serverUrl', 'http://localhost:3000');
        this.apiToken = config.get('apiToken', '');
    }

    private get headers(): Record<string, string> {
        return {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${this.apiToken}`,
        };
    }

    async getProjects(): Promise<VelumProject[]> {
        const response = await fetch(`${this.serverUrl}/api/projects`, {
            headers: this.headers,
        });
        if (!response.ok) {
            throw new Error(`Failed to fetch projects: ${response.statusText}`);
        }
        return response.json();
    }

    async getTemplates(projectId?: number): Promise<VelumTemplate[]> {
        const url = projectId
            ? `${this.serverUrl}/api/project/${projectId}/templates`
            : `${this.serverUrl}/api/templates`;
        const response = await fetch(url, { headers: this.headers });
        if (!response.ok) {
            throw new Error(`Failed to fetch templates: ${response.statusText}`);
        }
        return response.json();
    }

    async runTask(templateId: number, projectId: number, args?: Record<string, unknown>): Promise<VelumTask> {
        const body: Record<string, unknown> = { template_id: templateId };
        if (args) {
            body.arguments = JSON.stringify(args);
        }
        const response = await fetch(`${this.serverUrl}/api/project/${projectId}/tasks`, {
            method: 'POST',
            headers: this.headers,
            body: JSON.stringify(body),
        });
        if (!response.ok) {
            throw new Error(`Failed to run task: ${response.statusText}`);
        }
        return response.json();
    }

    async getTaskLogs(taskId: number): Promise<string> {
        const response = await fetch(`${this.serverUrl}/api/project/0/tasks/${taskId}/output`, {
            headers: this.headers,
        });
        if (!response.ok) {
            throw new Error(`Failed to fetch task logs: ${response.statusText}`);
        }
        const data = await response.json();
        return data.map((line: { output: string }) => line.output).join('\n');
    }

    async verifyConnection(): Promise<boolean> {
        try {
            const response = await fetch(`${this.serverUrl}/api/health`, {
                headers: this.headers,
            });
            return response.ok;
        } catch {
            return false;
        }
    }
}
