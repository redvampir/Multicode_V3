export interface PluginInfo {
  name: string;
  version: string;
  enabled: boolean;
}

export async function initPluginManager(container: HTMLElement) {
  const res = await fetch('/plugins');
  if (!res.ok) return;
  const plugins: PluginInfo[] = await res.json();
  container.innerHTML = '';
  plugins.forEach(p => {
    const row = document.createElement('div');
    const label = document.createElement('label');
    const checkbox = document.createElement('input');
    checkbox.type = 'checkbox';
    checkbox.checked = p.enabled;
    checkbox.addEventListener('change', async () => {
      await fetch('/plugins', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: p.name, enabled: checkbox.checked })
      });
    });
    label.appendChild(checkbox);
    label.append(` ${p.name} (${p.version})`);
    row.appendChild(label);
    container.appendChild(row);
  });
}
