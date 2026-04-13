import * as vscode from 'vscode';
import { VelumApi, VelumTemplate } from './api';

const ANSIBLE_TASK_KEYS = [
    'name', 'hosts', 'become', 'become_user', 'connection',
    'tasks', 'pre_tasks', 'post_tasks', 'roles', 'vars',
    'vars_files', 'environment', 'serial', 'strategy',
];

const TERRAFORM_TASK_KEYS = [
    'terraform', 'required_version', 'required_providers',
    'provider', 'resource', 'variable', 'output', 'module',
    'data', 'locals',
];

const ANSIBLE_MODULES = [
    'command', 'shell', 'copy', 'template', 'file', 'lineinfile',
    'replace', 'blockinfile', 'yum', 'apt', 'pip', 'service',
    'systemd', 'user', 'group', 'git', 'docker_container',
    'k8s', 'uri', 'debug', 'set_fact', 'include_tasks',
    'import_tasks', 'include_role', 'import_role',
];

export class TemplateCompletionProvider implements vscode.CompletionItemProvider {
    private api: VelumApi;
    private templates: VelumTemplate[] = [];
    private lastFetch: number = 0;
    private readonly CACHE_TTL = 5 * 60 * 1000; // 5 minutes

    constructor(api: VelumApi) {
        this.api = api;
    }

    async provideCompletionItems(
        document: vscode.TextDocument,
        position: vscode.Position,
        _token: vscode.CancellationToken,
        _context: vscode.CompletionContext
    ): Promise<vscode.CompletionItem[]> {
        const line = document.lineAt(position.line).text;
        const items: vscode.CompletionItem[] = [];

        // Ansible playbook completion
        if (this.isYamlPlaybook(document)) {
            await this.ensureTemplatesLoaded();

            // Template names from Velum server
            if (this.templates.length > 0) {
                for (const tpl of this.templates) {
                    const item = new vscode.CompletionItem(tpl.name, vscode.CompletionItemKind.Snippet);
                    item.detail = `Velum Template — ${tpl.type}`;
                    item.documentation = new vscode.MarkdownString(
                        `**${tpl.name}**\n\n${tpl.description || 'No description'}\n\nPlaybook: \`${tpl.playbook}\``
                    );
                    item.insertText = new vscode.SnippetString(`# Velum template: ${tpl.name}\n`);
                    items.push(item);
                }
            }

            // Ansible task keys
            if (line.match(/^\s{0,4}-?\s*$/)) {
                for (const key of ANSIBLE_TASK_KEYS) {
                    items.push(this.makeKeyItem(key));
                }
            }

            // Ansible modules
            if (line.match(/^\s{6,}(\w+):\s*$/)) {
                for (const mod of ANSIBLE_MODULES) {
                    items.push(this.makeModuleItem(mod));
                }
            }
        }

        // Terraform completion
        if (document.languageId === 'terraform' || document.fileName.endsWith('.tf')) {
            for (const key of TERRAFORM_TASK_KEYS) {
                items.push(this.makeTerraformKeyItem(key));
            }
        }

        return items;
    }

    private isYamlPlaybook(document: vscode.TextDocument): boolean {
        if (document.languageId !== 'yaml') return false;
        const text = document.getText(new vscode.Range(0, 0, 5, 0));
        return text.includes('hosts') || text.includes('name') || text.includes('tasks');
    }

    private async ensureTemplatesLoaded(): Promise<void> {
        if (this.templates.length > 0 && Date.now() - this.lastFetch < this.CACHE_TTL) {
            return;
        }
        try {
            const config = vscode.workspace.getConfiguration('velum');
            const projectId = config.get<number | null>('projectId', null);
            this.templates = await this.api.getTemplates(projectId || undefined);
            this.lastFetch = Date.now();
        } catch {
            // Silently fail — templates are optional
        }
    }

    private makeKeyItem(key: string): vscode.CompletionItem {
        const item = new vscode.CompletionItem(key, vscode.CompletionItemKind.Keyword);
        item.insertText = new vscode.SnippetString(`${key}: \${0}`);
        item.detail = 'Ansible task key';
        return item;
    }

    private makeModuleItem(mod: string): vscode.CompletionItem {
        const item = new vscode.CompletionItem(mod, vscode.CompletionItemKind.Class);
        item.insertText = new vscode.SnippetString(`${mod}:\n  \${0}`);
        item.detail = `Ansible module: ${mod}`;
        return item;
    }

    private makeTerraformKeyItem(key: string): vscode.CompletionItem {
        const item = new vscode.CompletionItem(key, vscode.CompletionItemKind.Keyword);
        item.insertText = new vscode.SnippetString(`${key} "\${1:name}" {\n  \${0}\n}`);
        item.detail = 'Terraform block';
        return item;
    }
}
