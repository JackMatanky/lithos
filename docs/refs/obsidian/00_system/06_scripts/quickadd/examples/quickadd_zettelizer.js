module.exports = async (e) => {
  console.log('Starting...'), console.log(e);
  const o = e.app.workspace.getActiveFile();
  if (!o) return void new Notice('No active file.');
  console.log('Found active file: ', o.basename);
  const a = e.app.metadataCache.getFileCache(o).headings;
  if (!a) return void new Notice(`No headers in file ${o.name}`);
  console.log('Found headings in active file: ', a);
  const i = '40 Slipbox/44 Zettels';
  e.app.vault.adapter.exists(i)
    ? (console.log('Folder does exist: ', i),
      a.forEach(async (a) => {
        if (
          (console.log(`Checking ${a.heading}. It is level ${a.level}`),
          3 === a.level)
        ) {
          const t = a.heading.split(' '),
            l = t[0].trim(),
            n = t.length > 1 ? [...t.slice(1)].join(' ').trim() : '',
            s = `${i}/${n.replace(/[\\,#%&\{\}\/*<>$\'\":@]*/g, '')}.md`,
            c = `![[${o.basename}#${l}${n ? ' ' + n : ''}]]`;
          console.log(`Path: ${s}.\nContent: ${c}`),
            n && !(await e.app.vault.adapter.exists(s))
              ? await e.app.vault.create(s, c)
              : n && new Notice(`File ${s} already exists.`, 5e3);
        }
      }),
      console.log('Finished!'))
    : new Notice(`Could not find folder ${i}`);
};
