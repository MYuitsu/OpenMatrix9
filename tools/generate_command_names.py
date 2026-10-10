"""Generate packaged original command names from local spec frontmatter."""
from pathlib import Path
import re

def generate(root):
    names={}
    for path in sorted((root/'ref/matrix9/OpenMatrix9_Codex_Spec_v1/specs').glob('*/*.md')):
        text=path.read_text(encoding='utf-8-sig')
        if not text.startswith('---'):continue
        front=text.split('---',2)[1]
        match=re.search(r'^command:\s*(.+)$',front,re.M)
        if not match:continue
        value=match.group(1).strip().strip('"').strip("'")
        if value=='null' or not re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*(?: [A-Za-z][A-Za-z0-9_]*)*',value):continue
        names.setdefault(value.casefold(),value)
    target=root/'Resources/menu/CommandNames.txt'
    target.write_text('# Original command names from local spec frontmatter; unavailable names are not executable.\n'+'\n'.join(sorted(names.values(),key=str.casefold))+'\n',encoding='utf-8')
    return len(names)
if __name__=='__main__':print('Packaged command names:',generate(Path(__file__).resolve().parents[1]))
