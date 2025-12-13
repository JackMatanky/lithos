---
title: venv_Creation of Virtual Environments
uuid: 8181bdea-d239-4b5a-803a-c5ff9346ebb7
main_title: venv
subtitle: Creation of Virtual Environments
author:
editor:
date_published:
publisher: python_software_foundation
url: https://docs.python.org/3/library/venv.html
cssclasses: null
status: resource
application: python
type: documentation
file_class: lib_documentation
date_created: 2023-07-13T14:07
date_modified: 2025-10-05T14:36
tags:
aliases:
  - "Venv: Creation of Virtual Environments"
---
# Venv: Creation of Virtual Environments

> [!webpage] Details
>
> - **Author**:: [[python_software_foundation|Python Software Foundation]]
> Title: [venv — Creation of virtual environments](https://docs.python.org/3/library/venv.html)
> - **Date Published**::
>
> Completed::

---

New in version 3.3.

**Source code:** [Lib/venv/](https://github.com/python/cpython/tree/3.11/Lib/venv/)

---

The `venv` module supports creating lightweight "virtual environments", each with their own independent set of Python packages installed in their [`site`](https://docs.python.org/3/library/site.html#module-site "site: Module responsible for site-specific configuration.") directories. A virtual environment is created on top of an existing Python installation, known as the virtual environment's "base" Python, and may optionally be isolated from the packages in the base environment, so only those explicitly installed in the virtual environment are available.

When used from within a virtual environment, common installation tools such as [pip](https://pypi.org/project/pip/) will install Python packages into a virtual environment without needing to be told to do so explicitly.

See [**PEP 405**](https://peps.python.org/pep-0405/) for more background on Python virtual environments.

[Availability](https://docs.python.org/3/library/intro.html#availability): not Emscripten, not WASI.

This module does not work or is not available on WebAssembly platforms `wasm32-emscripten` and `wasm32-wasi`. See [WebAssembly platforms](https://docs.python.org/3/library/intro.html#wasm-availability) for more information.

## Creating Virtual Environments

Creation of [virtual environments](https://docs.python.org/3/library/venv.html#venv-def) is done by executing the command `venv`:

```python
python -m venv /path/to/new/virtual/environment

```

Running this command creates the target directory (creating any parent directories that don't exist already) and places a `pyvenv.cfg` file in it with a `home` key pointing to the Python installation from which the command was run (a common name for the target directory is `.venv`). It also creates a `bin` (or `Scripts` on Windows) subdirectory containing a copy/symlink of the Python binary/binaries (as appropriate for the platform or arguments used at environment creation time). It also creates an (initially empty) `lib/pythonX.Y/site-packages` subdirectory (on Windows, this is `Lib\site-packages`). If an existing directory is specified, it will be re-used.

Deprecated since version 3.6: `pyvenv` was the recommended tool for creating virtual environments for Python 3.3 and 3.4, and is [deprecated in Python 3.6](https://docs.python.org/3/whatsnew/3.6.html#whatsnew36-venv).

Changed in version 3.5: The use of `venv` is now recommended for creating virtual environments.

On Windows, invoke the `venv` command as follows:

```python
c:\>c:\Python35\python -m venv c:\path\to\myenv

```

Alternatively, if you configured the `PATH` and `PATHEXT` variables for your [Python installation](https://docs.python.org/3/using/windows.html#using-on-windows):

```python
c:\>python -m venv c:\path\to\myenv

```

The command, if run with `-h`, will show the available options:

```python
usage: venv [-h] [--system-site-packages] [--symlinks | --copies] [--clear]
            [--upgrade] [--without-pip] [--prompt PROMPT] [--upgrade-deps]
            ENV_DIR [ENV_DIR ...]

Creates virtual Python environments in one or more target directories.

positional arguments:
  ENV_DIR               A directory to create the environment in.

optional arguments:
  -h, --help            show this help message and exit
  --system-site-packages
                        Give the virtual environment access to the system
                        site-packages dir.
  --symlinks            Try to use symlinks rather than copies, when symlinks
                        are not the default for the platform.
  --copies              Try to use copies rather than symlinks, even when
                        symlinks are the default for the platform.
  --clear               Delete the contents of the environment directory if it
                        already exists, before environment creation.
  --upgrade             Upgrade the environment directory to use this version
                        of Python, assuming Python has been upgraded in-place.
  --without-pip         Skips installing or upgrading pip in the virtual
                        environment (pip is bootstrapped by default)
  --prompt PROMPT       Provides an alternative prompt prefix for this
                        environment.
  --upgrade-deps        Upgrade core dependencies: pip setuptools to the
                        latest version in PyPI

Once an environment has been created, you may wish to activate it, e.g. by
sourcing an activate script in its bin directory.

```

Changed in version 3.9: Add `--upgrade-deps` option to upgrade pip + setuptools to the latest on PyPI

Changed in version 3.4: Installs pip by default, added the `--without-pip` and `--copies` options

Changed in version 3.4: In earlier versions, if the target directory already existed, an error was raised, unless the `--clear` or `--upgrade` option was provided.

Note

While symlinks are supported on Windows, they are not recommended. Of particular note is that double-clicking `python.exe` in File Explorer will resolve the symlink eagerly and ignore the virtual environment.

Note

On Microsoft Windows, it may be required to enable the `Activate.ps1` script by setting the execution policy for the user. You can do this by issuing the following PowerShell command:

PS C:> Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser

See [About Execution Policies](https://go.microsoft.com/fwlink/?LinkID=135170) for more information.

The created `pyvenv.cfg` file also includes the `include-system-site-packages` key, set to `true` if `venv` is run with the `--system-site-packages` option, `false` otherwise.

Unless the `--without-pip` option is given, [`ensurepip`](<https://docs.python.org/3/library/ensurepip.html#module-ensurepip> "ensurepip: Bootstrapping the "pip" installer into an existing Python installation or virtual environment.") will be invoked to bootstrap `pip` into the virtual environment.

Multiple paths can be given to `venv`, in which case an identical virtual environment will be created, according to the given options, at each provided path.

## How Venvs work[¶](https://docs.python.org/3/library/venv.html#how-venvs-work "Permalink to this headline")

When a Python interpreter is running from a virtual environment, [`sys.prefix`](https://docs.python.org/3/library/sys.html#sys.prefix "sys.prefix") and [`sys.exec_prefix`](https://docs.python.org/3/library/sys.html#sys.exec_prefix "sys.exec_prefix") point to the directories of the virtual environment, whereas [`sys.base_prefix`](https://docs.python.org/3/library/sys.html#sys.base_prefix "sys.base_prefix") and [`sys.base_exec_prefix`](https://docs.python.org/3/library/sys.html#sys.base_exec_prefix "sys.base_exec_prefix") point to those of the base Python used to create the environment. It is sufficient to check `sys.prefix!= sys.base_prefix` to determine if the current interpreter is running from a virtual environment.

A virtual environment may be "activated" using a script in its binary directory (`bin` on POSIX; `Scripts` on Windows). This will prepend that directory to your `PATH`, so that running **python** will invoke the environment's Python interpreter and you can run installed scripts without having to use their full path. The invocation of the activation script is platform-specific (`*<venv>*` must be replaced by the path to the directory containing the virtual environment):

|
Platform

 |

Shell

 |

Command to activate virtual environment

 |
| --- | --- | --- |
|

POSIX

 |

bash/zsh

 |

`$ source *<venv>*/bin/activate`

 |
|

fish

 |

`$ source *<venv>*/bin/activate.fish`

 |
|

csh/tcsh

 |

`$ source *<venv>*/bin/activate.csh`

 |
|

PowerShell

 |

`$ *<venv>*/bin/Activate.ps1`

 |
|

Windows

 |

cmd.exe

 |

`C:\> *<venv>*\Scripts\activate.bat`

 |
|

PowerShell

 |

`PS C:\> *<venv>*\Scripts\Activate.ps1`

 |

New in version 3.4: **fish** and **csh** activation scripts.

New in version 3.8: PowerShell activation scripts installed under POSIX for PowerShell Core support.

You don't specifically *need* to activate a virtual environment, as you can just specify the full path to that environment's Python interpreter when invoking Python. Furthermore, all scripts installed in the environment should be runnable without activating it.

In order to achieve this, scripts installed into virtual environments have a "shebang" line which points to the environment's Python interpreter, i.e. `#!/*<path-to-venv>*/bin/python`. This means that the script will run with that interpreter regardless of the value of `PATH`. On Windows, "shebang" line processing is supported if you have the [Python Launcher for Windows](https://docs.python.org/3/using/windows.html#launcher) installed. Thus, double-clicking an installed script in a Windows Explorer window should run it with the correct interpreter without the environment needing to be activated or on the `PATH`.

When a virtual environment has been activated, the `VIRTUAL_ENV` environment variable is set to the path of the environment. Since explicitly activating a virtual environment is not required to use it, `VIRTUAL_ENV` cannot be relied upon to determine whether a virtual environment is being used.

Warning

Because scripts installed in environments should not expect the environment to be activated, their shebang lines contain the absolute paths to their environment's interpreters. Because of this, environments are inherently non-portable, in the general case. You should always have a simple means of recreating an environment (for example, if you have a requirements file `requirements.txt`, you can invoke `pip install -r requirements.txt` using the environment's `pip` to install all of the packages needed by the environment). If for any reason you need to move the environment to a new location, you should recreate it at the desired location and delete the one at the old location. If you move an environment because you moved a parent directory of it, you should recreate the environment in its new location. Otherwise, software installed into the environment may not work as expected.

You can deactivate a virtual environment by typing `deactivate` in your shell. The exact mechanism is platform-specific and is an internal implementation detail (typically, a script or shell function will be used).

## API[¶](https://docs.python.org/3/library/venv.html#api "Permalink to this headline")

The high-level method described above makes use of a simple API which provides mechanisms for third-party virtual environment creators to customize environment creation according to their needs, the [`EnvBuilder`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder "venv.EnvBuilder") class.

*class* venv.EnvBuilder(*system\_site\_packages\=False*, *clear\=False*, *symlinks\=False*, *upgrade\=False*, *with\_pip\=False*, *prompt\=None*, *upgrade\_deps\=False*)[¶](https://docs.python.org/3/library/venv.html#venv.EnvBuilder "Permalink to this definition")

The [`EnvBuilder`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder "venv.EnvBuilder") class accepts the following keyword arguments on instantiation:

- `system_site_packages` – a Boolean value indicating that the system Python site-packages should be available to the environment (defaults to `False`).
- `clear` – a Boolean value which, if true, will delete the contents of any existing target directory, before creating the environment.
- `symlinks` – a Boolean value indicating whether to attempt to symlink the Python binary rather than copying.
- `upgrade` – a Boolean value which, if true, will upgrade an existing environment with the running Python - for use when that Python has been upgraded in-place (defaults to `False`).
- `with_pip` – a Boolean value which, if true, ensures pip is installed in the virtual environment. This uses [`ensurepip`](<https://docs.python.org/3/library/ensurepip.html#module-ensurepip> "ensurepip: Bootstrapping the "pip" installer into an existing Python installation or virtual environment.") with the `--default-pip` option.
- `prompt` – a String to be used after virtual environment is activated (defaults to `None` which means directory name of the environment would be used). If the special string `"."` is provided, the basename of the current directory is used as the prompt.
- `upgrade_deps` – Update the base venv modules to the latest on PyPI

Changed in version 3.4: Added the `with_pip` parameter

New in version 3.6: Added the `prompt` parameter

New in version 3.9: Added the `upgrade_deps` parameter

Creators of third-party virtual environment tools will be free to use the provided [`EnvBuilder`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder "venv.EnvBuilder") class as a base class.

The returned env-builder is an object which has a method, `create`:

create(*env\_dir*)[¶](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.create "Permalink to this definition")

Create a virtual environment by specifying the target directory (absolute or relative to the current directory) which is to contain the virtual environment. The `create` method will either create the environment in the specified directory, or raise an appropriate exception.

The `create` method of the [`EnvBuilder`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder "venv.EnvBuilder") class illustrates the hooks available for subclass customization:

```python
def create(self, env_dir):
    """
    Create a virtualized Python environment in a directory.
    env_dir is the target directory to create an environment in.
    """
    env_dir = os.path.abspath(env_dir)
    context = self.ensure_directories(env_dir)
    self.create_configuration(context)
    self.setup_python(context)
    self.setup_scripts(context)
    self.post_setup(context)

```

Each of the methods [`ensure_directories()`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.ensure_directories "venv.EnvBuilder.ensure_directories"), [`create_configuration()`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.create_configuration "venv.EnvBuilder.create_configuration"), [`setup_python()`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.setup_python "venv.EnvBuilder.setup_python"), [`setup_scripts()`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.setup_scripts "venv.EnvBuilder.setup_scripts") and [`post_setup()`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.post_setup "venv.EnvBuilder.post_setup") can be overridden.

ensure\_directories(*env\_dir*)[¶](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.ensure_directories "Permalink to this definition")

Creates the environment directory and all necessary subdirectories that don't already exist, and returns a context object. This context object is just a holder for attributes (such as paths) for use by the other methods. If the [`EnvBuilder`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder "venv.EnvBuilder") is created with the arg `clear=True`, contents of the environment directory will be cleared and then all necessary subdirectories will be recreated.

The returned context object is a [`types.SimpleNamespace`](https://docs.python.org/3/library/types.html#types.SimpleNamespace "types.SimpleNamespace") with the following attributes:

- `env_dir` - The location of the virtual environment. Used for `__VENV_DIR__` in activation scripts (see [`install_scripts()`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.install_scripts "venv.EnvBuilder.install_scripts")).
- `env_name` - The name of the virtual environment. Used for `__VENV_NAME__` in activation scripts (see [`install_scripts()`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.install_scripts "venv.EnvBuilder.install_scripts")).
- `prompt` - The prompt to be used by the activation scripts. Used for `__VENV_PROMPT__` in activation scripts (see [`install_scripts()`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.install_scripts "venv.EnvBuilder.install_scripts")).
- `executable` - The underlying Python executable used by the virtual environment. This takes into account the case where a virtual environment is created from another virtual environment.
- `inc_path` - The include path for the virtual environment.
- `lib_path` - The purelib path for the virtual environment.
- `bin_path` - The script path for the virtual environment.
- `bin_name` - The name of the script path relative to the virtual environment location. Used for `__VENV_BIN_NAME__` in activation scripts (see [`install_scripts()`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.install_scripts "venv.EnvBuilder.install_scripts")).
- `env_exe` - The name of the Python interpreter in the virtual environment. Used for `__VENV_PYTHON__` in activation scripts (see [`install_scripts()`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.install_scripts "venv.EnvBuilder.install_scripts")).
- `env_exec_cmd` - The name of the Python interpreter, taking into account filesystem redirections. This can be used to run Python in the virtual environment.

Changed in version 3.12: The attribute `lib_path` was added to the context, and the context object was documented.

Changed in version 3.11: The *venv* [sysconfig installation scheme](https://docs.python.org/3/library/sysconfig.html#installation-paths) is used to construct the paths of the created directories.

create\_configuration(*context*)[¶](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.create_configuration "Permalink to this definition")

Creates the `pyvenv.cfg` configuration file in the environment.

setup\_python(*context*)[¶](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.setup_python "Permalink to this definition")

Creates a copy or symlink to the Python executable in the environment. On POSIX systems, if a specific executable `python3.x` was used, symlinks to `python` and `python3` will be created pointing to that executable, unless files with those names already exist.

setup\_scripts(*context*)[¶](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.setup_scripts "Permalink to this definition")

Installs activation scripts appropriate to the platform into the virtual environment.

upgrade\_dependencies(*context*)[¶](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.upgrade_dependencies "Permalink to this definition")

Upgrades the core venv dependency packages (currently `pip` and `setuptools`) in the environment. This is done by shelling out to the `pip` executable in the environment.

New in version 3.9.

post\_setup(*context*)[¶](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.post_setup "Permalink to this definition")

A placeholder method which can be overridden in third party implementations to pre-install packages in the virtual environment or perform other post-creation steps.

Changed in version 3.7.2: Windows now uses redirector scripts for `python[w].exe` instead of copying the actual binaries. In 3.7.2 only [`setup_python()`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.setup_python "venv.EnvBuilder.setup_python") does nothing unless running from a build in the source tree.

Changed in version 3.7.3: Windows copies the redirector scripts as part of [`setup_python()`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.setup_python "venv.EnvBuilder.setup_python") instead of [`setup_scripts()`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.setup_scripts "venv.EnvBuilder.setup_scripts"). This was not the case in 3.7.2. When using symlinks, the original executables will be linked.

In addition, [`EnvBuilder`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder "venv.EnvBuilder") provides this utility method that can be called from [`setup_scripts()`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.setup_scripts "venv.EnvBuilder.setup_scripts") or [`post_setup()`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.post_setup "venv.EnvBuilder.post_setup") in subclasses to assist in installing custom scripts into the virtual environment.

install\_scripts(*context*, *path*)[¶](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.install_scripts "Permalink to this definition")

*path* is the path to a directory that should contain subdirectories "common", "posix", "nt", each containing scripts destined for the bin directory in the environment. The contents of "common" and the directory corresponding to [`os.name`](https://docs.python.org/3/library/os.html#os.name "os.name") are copied after some text replacement of placeholders:

- `__VENV_DIR__` is replaced with the absolute path of the environment directory.
- `__VENV_NAME__` is replaced with the environment name (final path segment of environment directory).
- `__VENV_PROMPT__` is replaced with the prompt (the environment name surrounded by parentheses and with a following space)
- `__VENV_BIN_NAME__` is replaced with the name of the bin directory (either `bin` or `Scripts`).
- `__VENV_PYTHON__` is replaced with the absolute path of the environment's executable.

The directories are allowed to exist (for when an existing environment is being upgraded).

There is also a module-level convenience function:

venv.create(*env\_dir*, *system\_site\_packages\=False*, *clear\=False*, *symlinks\=False*, *with\_pip\=False*, *prompt\=None*, *upgrade\_deps\=False*)[¶](https://docs.python.org/3/library/venv.html#venv.create "Permalink to this definition")

Create an [`EnvBuilder`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder "venv.EnvBuilder") with the given keyword arguments, and call its [`create()`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder.create "venv.EnvBuilder.create") method with the *env\_dir* argument.

New in version 3.3.

Changed in version 3.4: Added the `with_pip` parameter

Changed in version 3.6: Added the `prompt` parameter

Changed in version 3.9: Added the `upgrade_deps` parameter

## An Example of Extending `EnvBuilder`[¶](https://docs.python.org/3/library/venv.html#an-example-of-extending-envbuilder "Permalink to this headline")

The following script shows how to extend [`EnvBuilder`](https://docs.python.org/3/library/venv.html#venv.EnvBuilder "venv.EnvBuilder") by implementing a subclass which installs setuptools and pip into a created virtual environment:

```python
import os
import os.path
from subprocess import Popen, PIPE
import sys
from threading import Thread
from urllib.parse import urlparse
from urllib.request import urlretrieve
import venv

class ExtendedEnvBuilder(venv.EnvBuilder):
    """
    This builder installs setuptools and pip so that you can pip or
    easy_install other packages into the created virtual environment.

    :param nodist: If true, setuptools and pip are not installed into the
                   created virtual environment.
    :param nopip: If true, pip is not installed into the created
                  virtual environment.
    :param progress: If setuptools or pip are installed, the progress of the
                     installation can be monitored by passing a progress
                     callable. If specified, it is called with two
                     arguments: a string indicating some progress, and a
                     context indicating where the string is coming from.
                     The context argument can have one of three values:
                     'main', indicating that it is called from virtualize()
                     itself, and 'stdout' and 'stderr', which are obtained
                     by reading lines from the output streams of a subprocess
                     which is used to install the app.

                     If a callable is not specified, default progress
                     information is output to sys.stderr.
    """

    def __init__(self, *args, **kwargs):
        self.nodist = kwargs.pop('nodist', False)
        self.nopip = kwargs.pop('nopip', False)
        self.progress = kwargs.pop('progress', None)
        self.verbose = kwargs.pop('verbose', False)
        super().__init__(*args, **kwargs)

    def post_setup(self, context):
        """
        Set up any packages which need to be pre-installed into the
        virtual environment being created.

        :param context: The information for the virtual environment
                        creation request being processed.
        """
        os.environ['VIRTUAL_ENV'] = context.env_dir
        if not self.nodist:
            self.install_setuptools(context)
        # Can't install pip without setuptools
        if not self.nopip and not self.nodist:
            self.install_pip(context)

    def reader(self, stream, context):
        """
        Read lines from a subprocess' output stream and either pass to a progress
        callable (if specified) or write progress information to sys.stderr.
        """
        progress = self.progress
        while True:
            s = stream.readline()
            if not s:
                break
            if progress is not None:
                progress(s, context)
            else:
                if not self.verbose:
                    sys.stderr.write('.')
                else:
                    sys.stderr.write(s.decode('utf-8'))
                sys.stderr.flush()
        stream.close()

    def install_script(self, context, name, url):
        _, _, path, _, _, _ = urlparse(url)
        fn = os.path.split(path)[-1]
        binpath = context.bin_path
        distpath = os.path.join(binpath, fn)
        # Download script into the virtual environment's binaries folder
        urlretrieve(url, distpath)
        progress = self.progress
        if self.verbose:
            term = '\n'
        else:
            term = ''
        if progress is not None:
            progress('Installing %s ...%s' % (name, term), 'main')
        else:
            sys.stderr.write('Installing %s ...%s' % (name, term))
            sys.stderr.flush()
        # Install in the virtual environment
        args = [context.env_exe, fn]
        p = Popen(args, stdout=PIPE, stderr=PIPE, cwd=binpath)
        t1 = Thread(target=self.reader, args=(p.stdout, 'stdout'))
        t1.start()
        t2 = Thread(target=self.reader, args=(p.stderr, 'stderr'))
        t2.start()
        p.wait()
        t1.join()
        t2.join()
        if progress is not None:
            progress('done.', 'main')
        else:
            sys.stderr.write('done.\n')
        # Clean up - no longer needed
        os.unlink(distpath)

    def install_setuptools(self, context):
        """
        Install setuptools in the virtual environment.

        :param context: The information for the virtual environment
                        creation request being processed.
        """
        url = 'https://bitbucket.org/pypa/setuptools/downloads/ez_setup.py'
        self.install_script(context, 'setuptools', url)
        # clear up the setuptools archive which gets downloaded
        pred = lambda o: o.startswith('setuptools-') and o.endswith('.tar.gz')
        files = filter(pred, os.listdir(context.bin_path))
        for f in files:
            f = os.path.join(context.bin_path, f)
            os.unlink(f)

    def install_pip(self, context):
        """
        Install pip in the virtual environment.

        :param context: The information for the virtual environment
                        creation request being processed.
        """
        url = 'https://bootstrap.pypa.io/get-pip.py'
        self.install_script(context, 'pip', url)

def main(args=None):
    compatible = True
    if sys.version_info < (3, 3):
        compatible = False
    elif not hasattr(sys, 'base_prefix'):
        compatible = False
    if not compatible:
        raise ValueError('This script is only for use with '
                         'Python 3.3 or later')
    else:
        import argparse

        parser = argparse.ArgumentParser(prog=__name__,
                                         description='Creates virtual Python '
                                                     'environments in one or '
                                                     'more target '
                                                     'directories.')
        parser.add_argument('dirs', metavar='ENV_DIR', nargs='+',
                            help='A directory in which to create the '
                                 'virtual environment.')
        parser.add_argument('--no-setuptools', default=False,
                            action='store_true', dest='nodist',
                            help="Don't install setuptools or pip in the "
                                 "virtual environment.")
        parser.add_argument('--no-pip', default=False,
                            action='store_true', dest='nopip',
                            help="Don't install pip in the virtual "
                                 "environment.")
        parser.add_argument('--system-site-packages', default=False,
                            action='store_true', dest='system_site',
                            help='Give the virtual environment access to the '
                                 'system site-packages dir.')
        if os.name == 'nt':
            use_symlinks = False
        else:
            use_symlinks = True
        parser.add_argument('--symlinks', default=use_symlinks,
                            action='store_true', dest='symlinks',
                            help='Try to use symlinks rather than copies, '
                                 'when symlinks are not the default for '
                                 'the platform.')
        parser.add_argument('--clear', default=False, action='store_true',
                            dest='clear', help='Delete the contents of the '
                                               'virtual environment '
                                               'directory if it already '
                                               'exists, before virtual '
                                               'environment creation.')
        parser.add_argument('--upgrade', default=False, action='store_true',
                            dest='upgrade', help='Upgrade the virtual '
                                                 'environment directory to '
                                                 'use this version of '
                                                 'Python, assuming Python '
                                                 'has been upgraded '
                                                 'in-place.')
        parser.add_argument('--verbose', default=False, action='store_true',
                            dest='verbose', help='Display the output '
                                               'from the scripts which '
                                               'install setuptools and pip.')
        options = parser.parse_args(args)
        if options.upgrade and options.clear:
            raise ValueError('you cannot supply --upgrade and --clear together.')
        builder = ExtendedEnvBuilder(system_site_packages=options.system_site,
                                       clear=options.clear,
                                       symlinks=options.symlinks,
                                       upgrade=options.upgrade,
                                       nodist=options.nodist,
                                       nopip=options.nopip,
                                       verbose=options.verbose)
        for d in options.dirs:
            builder.create(d)

if __name__ == '__main__':
    rc = 1
    try:
        main()
        rc = 0
    except Exception as e:
        print('Error: %s' % e, file=sys.stderr)
    sys.exit(rc)

```

This script is also available for download [online](https://gist.github.com/vsajip/4673395).
