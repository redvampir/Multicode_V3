export interface PluginInfo {
  name: string;
  version: string;
  enabled: boolean;
}

export async function initPluginManager(container: HTMLElement) {
  const res = await fetch('/plugins');
  if (!res.ok) {
    alert('Failed to load plugins');
    return;
  }
  const plugins: PluginInfo[] = await res.json();
  container.innerHTML = '';
  plugins.forEach(p => {
    const row = document.createElement('div');
    const label = document.createElement('label');
    const checkbox = document.createElement('input');
    checkbox.type = 'checkbox';
    checkbox.checked = p.enabled;
    checkbox.addEventListener('change', async () => {
      const res = await fetch('/plugins', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name: p.name, enabled: checkbox.checked })
      });
      if (!res.ok) {
        alert('Failed to update plugin');
        checkbox.checked = !checkbox.checked;
      }
    });
    label.appendChild(checkbox);
    label.append(' ');
    const nameSpan = document.createElement('span');
    nameSpan.textContent = p.name;
    label.appendChild(nameSpan);
    const versionSpan = document.createElement('span');
    versionSpan.textContent = ' (' + p.version + ')';
    label.appendChild(versionSpan);
    row.appendChild(label);
    container.appendChild(row);
  });
}
