import * as vscode from 'vscode';
import { VelumApi } from './api';
import { TemplateCompletionProvider } from './templates';

let api: VelumApi;
let outputChannel: vscode.OutputChannel;

export function activate(context: vscode.ExtensionContext) {
    outputChannel = vscode.window.createOutputChannel('Velum');
    context.subscriptions.push(outputChannel);

    api = new VelumApi();

    // Register completion provider for templates
    const provider = new TemplateCompletionProvider(api);
    const yamlSelector: vscode.DocumentSelector = { language: 'yaml', scheme: 'file' };
    context.subscriptions.push(
        vscode.languages.registerCompletionItemProvider(yamlSelector, provider, ' ', ':', '-')
    );

    // Register commands
    context.subscriptions.push(
        vscode.commands.registerCommand('velum.login', cmdLogin),
        vscode.commands.registerCommand('velum.listProjects', cmdListProjects),
        vscode.commands.registerCommand('velum.listTemplates', cmdListTemplates),
        vscode.commands.registerCommand('velum.runTask', cmdRunTask),
        vscode.commands.registerCommand('velum.viewTaskLogs', cmdViewTaskLogs),
    );

    outputChannel.appendLine('Velum extension activated');
}

export function deactivate() {
    outputChannel.appendLine('Velum extension deactivated');
}

// ─── Commands ───────────────────────────────────────────────────────────────

async function cmdLogin() {
    const config = vscode.workspace.getConfiguration('velum');

    const serverUrl = await vscode.window.showInputBox({
        prompt: 'Velum server URL',
        value: config.get('serverUrl', 'http://localhost:3000'),
        placeHolder: 'http://localhost:3000',
    });
    if (!serverUrl) return;

    const apiToken = await vscode.window.showInputBox({
        prompt: 'Velum API token',
        password: true,
        placeHolder: 'Enter your API token',
    });
    if (!apiToken) return;

    await config.update('serverUrl', serverUrl, vscode.ConfigurationTarget.Global);
    await config.update('apiToken', apiToken, vscode.ConfigurationTarget.Global);

    // Verify connection
    const testApi = new VelumApi();
    const connected = await testApi.verifyConnection();
    if (connected) {
        vscode.window.showInformationMessage(`Connected to Velum: ${serverUrl}`);
    } else {
        vscode.window.showWarningMessage(`Could not connect to ${serverUrl}. Check URL and token.`);
    }
}

async function cmdListProjects() {
    try {
        const projects = await api.getProjects();
        if (projects.length === 0) {
            vscode.window.showInformationMessage('No projects found');
            return;
        }
        const items = projects.map(p => ({
            label: `$(folder) ${p.name}`,
            description: `ID: ${p.id}`,
            detail: `Created: ${p.created}`,
            project: p,
        }));
        const selected = await vscode.window.showQuickPick(items, {
            placeHolder: 'Select a project',
        });
        if (selected) {
            await vscode.workspace.getConfiguration('velum').update(
                'projectId', selected.project.id, vscode.ConfigurationTarget.Global
            );
            vscode.window.showInformationMessage(`Project set: ${selected.project.name} (ID: ${selected.project.id})`);
        }
    } catch (err: unknown) {
        vscode.window.showErrorMessage(`Failed to list projects: ${(err as Error).message}`);
    }
}

async function cmdListTemplates() {
    try {
        const config = vscode.workspace.getConfiguration('velum');
        const projectId = config.get<number | null>('projectId', null);
        if (!projectId) {
            vscode.window.showWarningMessage('No project selected. Run "Velum: List Projects" first.');
            return;
        }
        const templates = await api.getTemplates(projectId);
        if (templates.length === 0) {
            vscode.window.showInformationMessage('No templates found in this project');
            return;
        }
        const items = templates.map(t => ({
            label: `$(file-code) ${t.name}`,
            description: `Type: ${t.type} | App: ${t.app || 'N/A'}`,
            detail: t.description || t.playbook,
            template: t,
        }));
        const selected = await vscode.window.showQuickPick(items, {
            placeHolder: 'Select a template',
        });
        if (selected) {
            vscode.env.clipboard.writeText(JSON.stringify(selected.template, null, 2));
            vscode.window.showInformationMessage(`Template details copied to clipboard: ${selected.template.name}`);
        }
    } catch (err: unknown) {
        vscode.window.showErrorMessage(`Failed to list templates: ${(err as Error).message}`);
    }
}

async function cmdRunTask() {
    try {
        const config = vscode.workspace.getConfiguration('velum');
        const projectId = config.get<number | null>('projectId', null);
        if (!projectId) {
            vscode.window.showWarningMessage('No project selected. Run "Velum: List Projects" first.');
            return;
        }
        const templates = await api.getTemplates(projectId);
        if (templates.length === 0) {
            vscode.window.showInformationMessage('No templates available');
            return;
        }
        const items = templates.map(t => ({
            label: `$(play) ${t.name}`,
            description: t.type,
            template: t,
        }));
        const selected = await vscode.window.showQuickPick(items, {
            placeHolder: 'Select template to run',
        });
        if (!selected) return;

        const task = await api.runTask(selected.template.id, projectId);
        vscode.window.showInformationMessage(`Task started: ID ${task.id} (Status: ${task.status})`);
        outputChannel.appendLine(`Task ${task.id} started from template "${selected.template.name}"`);
    } catch (err: unknown) {
        vscode.window.showErrorMessage(`Failed to run task: ${(err as Error).message}`);
    }
}

async function cmdViewTaskLogs() {
    try {
        const input = await vscode.window.showInputBox({
            prompt: 'Enter task ID',
            placeHolder: '123',
            validateInput: v => (v && /^\d+$/.test(v)) ? null : 'Enter a valid task ID (number)',
        });
        if (!input) return;

        const taskId = parseInt(input, 10);
        const logs = await api.getTaskLogs(taskId);

        outputChannel.clear();
        outputChannel.appendLine(`--- Logs for task #${taskId} ---`);
        outputChannel.appendLine(logs);
        outputChannel.show(true);
    } catch (err: unknown) {
        vscode.window.showErrorMessage(`Failed to fetch logs: ${(err as Error).message}`);
    }
}
