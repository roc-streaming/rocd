from doit.tools import title_with_actions, Interactive
import atexit
import functools
import glob
import os
import shlex
import shutil
import signal
import sys

atexit.register(
    lambda: shutil.rmtree('__pycache__', ignore_errors=True))
if os.name == 'posix':
    signal.signal(signal.SIGINT, lambda s, f: exit(1))

sys.stdin.reconfigure(encoding='utf-8')
sys.stdout.reconfigure(encoding='utf-8')

os.environ['TERM'] = 'xterm-color'

def _delete_files(pattern):
    def task():
        for path in glob.glob(pattern):
            print(f"Removing '{path}'")
            if os.path.isdir(path):
                shutil.rmtree(path)
            else:
                os.remove(path)
    return task

@functools.cache
def _colors_supported():
    try:
        import colorama
        if sys.stderr.isatty():
            colorama.init(autoreset=False)
            return True
    except ImportError:
        pass
    return False

@functools.cache
def _colorize(text, color):
    if _colors_supported():
        import colorama
        text = getattr(colorama.Fore, color.upper(), '') + \
            text + colorama.Style.RESET_ALL
    return text

def _color_title(task):
    text = title_with_actions(task)
    if '=>' in text:
        name, title = text.split('=>')
        return '{}=>{}'.format(_colorize(name, 'yellow'), title)
    else:
        return _colorize(text, 'yellow')

DOIT_CONFIG = {
    'default_tasks': ['all'],
    'verbosity': 2,
}

# doit all
def task_all():
    """default target"""
    return {
        'basename': 'all',
        'actions': [],
        'task_dep': ['build', 'lint', 'test'],
        'title': _color_title,
    }

# doit build
def task_build():
    """build daemon and openapi spec"""
    return {
        'basename': 'build',
        'actions': [],
        'task_dep': ['build_exe', 'build_spec'],
        'title': _color_title,
    }

# doit build_exe
def task_build_exe():
    """build daemon"""
    return {
        'basename': 'build_exe',
        'actions': [
            Interactive(
                'cargo build',
                env={**os.environ, **{'RUSTFLAGS': '--deny warnings'}}),
        ],
        'title': _color_title,
    }

# doit build_spec
def task_build_spec():
    """build openapi spec"""
    return {
        'basename': 'build_spec',
        'actions': [
            Interactive('cargo run -- --dump-openapi=json > openapi/openapi.json'),
            Interactive('cargo run -- --dump-openapi=yaml > openapi/openapi.yaml'),
        ],
        'task_dep': ['build_exe'],
        'title': _color_title,
    }

# doit lint
def task_lint():
    """run clippy"""
    return {
        'basename': 'lint',
        'actions': [
            Interactive('cargo clippy'),
        ],
        'task_dep': ['build'],
        'title': _color_title,
    }

# doit test
def task_test():
    """run tests"""
    return {
        'basename': 'test',
        'actions': [
            Interactive('cargo test'),
        ],
        'task_dep': ['lint'],
        'title': _color_title,
    }

# doit fmt
def task_fmt():
    """run rustfmt"""
    return {
        'basename': 'fmt',
        'actions': [
            Interactive('cargo fmt'),
        ],
        'title': _color_title,
    }

# doit docs
def task_docs():
    """build all documentation"""
    return {
        'basename': 'docs',
        'actions': [],
        'task_dep': ['docs_d2', 'docs_api', 'docs_site'],
        'title': _color_title,
    }

# doit docs_d2
def task_docs_d2():
    """build svg files from d2 diagrams"""
    for d2_file in glob.glob('docs/assets/dia/*.d2'):
        svg_file = d2_file.replace('.d2', '.svg')
        yield {
            'basename': 'docs_d2',
            'name': d2_file,
            'file_dep': [d2_file],
            'targets': [svg_file],
            'actions': [f'd2 --theme 0 --dark-theme 200 --pad 5 --scale 0.98 {d2_file}'],
            'title': _color_title,
        }

# doit docs_api
def task_docs_api():
    """build openapi.html with redocly"""
    return {
        'basename': 'docs_api',
        'actions': [Interactive(
            # openapi tool from @redocly/cli
            'npm exec @redocly/cli -- build-docs -o openapi/openapi.html openapi/openapi.json')],
        'file_dep': ['openapi/openapi.json'],
        'targets': ['openapi/openapi.html'],
        'title': _color_title,
    }

# doit docs_site
def task_docs_site():
    """build html documentation with mkdocs"""
    return {
        'basename': 'docs_site',
        'actions': [Interactive('mkdocs build')],
        'task_dep': ['docs_d2', 'docs_api'],
        'title': _color_title,
    }

# doit wipe
def task_wipe():
    """remove build artifacts"""
    return {
        'basename': 'wipe',
        'actions': [
            _delete_files('target'),
        ],
        'title': _color_title,
    }

# doit wipe_docs
def task_wipe_docs():
    """remove documentation artifacts"""
    return {
        'basename': 'wipe_docs',
        'actions': [
            _delete_files('site'),
            _delete_files('openapi/openapi.html'),
            _delete_files('docs/assets/dia/*.svg'),
        ],
        'title': _color_title,
    }

# doit wipe_all
def task_wipe_all():
    """remove all artifacts (build, documentation, rust analyzer)"""
    return {
        'basename': 'wipe_all',
        'actions': [
            _delete_files('.cache'),
        ],
        'task_dep': ['wipe', 'wipe_docs'],
        'title': _color_title,
    }
