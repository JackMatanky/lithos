---
title: cookiecutter_data_science_automated_project_structure_and_files
uuid: 769fd903-ec25-4f68-b544-6f0b95e47396
aliases:
  - "Cookiecutter Data Science: Automated Project Structure and Files"
  - Cookiecutter Data Science_Automated Project Structure and Files,
  - Cookiecutter_Data_Science_Automated_Project_Structure_and_Files
  - cookiecutter_data_science_automated_project_structure_and_files
main_title: Cookiecutter Data Science
subtitle: Automated Project Structure and Files
author:
editor:
date_published:
publisher:
url: http://drivendata.github.io/cookiecutter-data-science/
about:
status: resource
type: documentation
file_class: lib_documentation
cssclasses: null
date_created: 2023-11-16T23:15
date_modified: 2025-10-05T14:36
tags:
---
# Cookiecutter Data Science: Automated Project Structure and Files

> [!webpage] Details
>
> - **Author**: `dv: this.file.frontmatter.author`
> - **Publisher**: `dv: this.file.frontmatter.publisher`
> - **Date Published**: `dv: this.file.frontmatter.date_published`
> - **Link**: `dv: elink(this.file.frontmatter.url, this.file.frontmatter.title)`
>
> Completed::

---

*A logical, reasonably standardized, but flexible project structure for doing and sharing data science work.*

## Why Use This Project Structure?

> We're not talking about bikeshedding the indentation aesthetics or pedantic formatting standards — ultimately, data science code quality is about correctness and reproducibility.

When we think about data analysis, we often think just about the resulting reports, insights, or visualizations. While these end products are generally the main event, it's easy to focus on making the products *look nice* and ignore the *quality of the code that generates them*. Because these end products are created programmatically, **code quality is still important**! And we're not talking about bikeshedding the indentation aesthetics or pedantic formatting standards — ultimately, data science code quality is about correctness and reproducibility.

It's no secret that good analyses are often the result of very scattershot and serendipitous explorations. Tentative experiments and rapidly testing approaches that might not work out are all part of the process for getting to the good stuff, and there is no magic bullet to turn data exploration into a simple, linear progression.

That being said, once started it is not a process that lends itself to thinking carefully about the structure of your code or project layout, so it's best to start with a clean, logical structure and stick to it throughout. We think it's a pretty big win all around to use a fairly standardized setup like this one. Here's why:

### Other People Will Thank You

> Nobody sits around before creating a new Rails project to figure out where they want to put their views; they just run `rails new` to get a standard project skeleton like everybody else.

A well-defined, standard project structure means that a newcomer can begin to understand an analysis without digging in to extensive documentation. It also means that they don't necessarily have to read 100% of the code before knowing where to look for very specific things.

Well organized code tends to be self-documenting in that the organization itself provides context for your code without much overhead. People will thank you for this because they can:

- collaborate more easily with you on this analysis
- Learn from your analysis about the process and the domain
- Feel confident in the conclusions at which the analysis arrives

A good example of this can be found in any of the major web development frameworks like Django or Ruby on Rails. Nobody sits around before creating a new Rails project to figure out where they want to put their views; they just run `rails new` to get a standard project skeleton like everybody else. Because that default project structure is *logical* and *reasonably standard across most projects*, it is much easier for somebody who has never seen a particular project to figure out where they would find the various moving parts.

Another great example is the [Filesystem Hierarchy Standard](https://en.wikipedia.org/wiki/Filesystem_Hierarchy_Standard) for Unix-like systems. The `/etc` directory has a very specific purpose, as does the `/tmp` folder, and everybody (more or less) agrees to honor that social contract. That means a Red Hat user and an Ubuntu user both know roughly where to look for certain types of files, even when using each other's system — or any other standards-compliant system for that matter!

Ideally, that's how it should be when a colleague opens up your data science project.

### You Will Thank You

Ever tried to reproduce an analysis that you did a few months ago or even a few years ago? You may have written the code, but it's now impossible to decipher whether you should use `make_figures.py.old`, `make_figures_working.py` or `new_make_figures01.py` to get things done. Here are some questions we've learned to ask with a sense of existential dread:

-re we supposed to go in and join the column X to the data before we get started or did that come from one of the notebooks?

- Come to think of it, which notebook do we have to run first before running the plotting code: was it "process data" or "clean data"?
- Where did the shapefiles get downloaded from for the geographic plots?
- *Et cetera, times infinity.*

These types of questions are painful and are symptoms of a disorganized project. A good project structure encourages practices that make it easier to come back to old work, for example separation of concerns, abstracting analysis as a [DAG](https://en.wikipedia.org/wiki/Directed_acyclic_graph), and engineering best practices like version control.

### Nothing here is Binding

> "A foolish consistency is the hobgoblin of little minds" — Ralph Waldo Emerson (and [PEP 8!](https://www.python.org/dev/peps/pep-0008/#a-foolish-consistency-is-the-hobgoblin-of-little-minds))

Disagree with a couple of the default folder names? Working on a project that's a little nonstandard and doesn't exactly fit with the current structure? Prefer to use a different package than one of the (few) defaults?

**Go for it!** This is a lightweight structure, and is intended to be a good *starting point* for many projects. Or, as PEP 8 put it:

> Consistency within a project is more important. Consistency within one module or function is the most importan….. However, know when to be inconsistent -- sometimes style guide recommendations just aren't applicable. When in doubt, use your best judgment. Look at other examples and decide what looks best. And don't hesitate to ask!

## Getting Started

With this in mind, we've created a data science cookiecutter template for projects in Python. Your analysis doesn't have to be in Python, but the template does provide some Python boilerplate that you'd want to remove (in the `src` folder for example, and the Sphinx documentation skeleton in `docs`).

### Requirements

- python 2.7 or 3.5
- [cookiecutter Python package](http://cookiecutter.readthedocs.org/en/latest/installation.html) >= 1.4.0: `pip install cookiecutter`

### Starting a New Project

Starting a new project is as easy as running this command at the command line. No need to create a directory first, the cookiecutter will do it for you.

```
cookiecutter https://github.com/drivendata/cookiecutter-data-science
```

### Example

## Directory Structure

```
├── LICENSE
├── Makefile           <- Makefile with commands like `make data` or `make train`
├── README.md          <- The top-level README for developers using this project.
├── data
│   ├── external       <- Data from third party sources.
│   ├── interim        <- Intermediate data that has been transformed.
│   ├── processed      <- The final, canonical data sets for modeling.
│   └── raw            <- The original, immutable data dump.
│
├── docs               <- A default Sphinx project; see sphinx-doc.org for details
│
├── models             <- Trained and serialized models, model predictions, or model summaries
│
├── notebooks          <- Jupyter notebooks. Naming convention is a number (for ordering),
│                         the creator's initials, and a short `-` delimited description, e.g.
│                         `1.0-jqp-initial-data-exploration`.
│
├── references         <- Data dictionaries, manuals, and all other explanatory materials.
│
├── reports            <- Generated analysis as HTML, PDF, LaTeX, etc.
│   └── figures        <- Generated graphics and figures to be used in reporting
│
├── requirements.txt   <- The requirements file for reproducing the analysis environment, e.g.
│                         generated with `pip freeze > requirements.txt`
│
├── setup.py           <- Make this project pip installable with `pip install -e`
├── src                <- Source code for use in this project.
│   ├── __init__.py    <- Makes src a Python module
│   │
│   ├── data           <- Scripts to download or generate data
│   │   └── make_dataset.py
│   │
│   ├── features       <- Scripts to turn raw data into features for modeling
│   │   └── build_features.py
│   │
│   ├── models         <- Scripts to train models and then use trained models to make
│   │   │                 predictions
│   │   ├── predict_model.py
│   │   └── train_model.py
│   │
│   └── visualization  <- Scripts to create exploratory and results oriented visualizations
│       └── visualize.py
│
└── tox.ini            <- tox file with settings for running tox; see tox.readthedocs.io
```

## Opinions

There are some opinions implicit in the project structure that have grown out of our experience with what works and what doesn't when collaborating on data science projects. Some of the opinions are about workflows, and some of the opinions are about tools that make life easier. Here are some of the beliefs which this project is built on—if you've got thoughts, please [contribute or share them](http://drivendata.github.io/cookiecutter-data-science/#contributing).

### Data is Immutable

Don't ever edit your raw data, especially not manually, and especially not in Excel. Don't overwrite your raw data. Don't save multiple versions of the raw data. Treat the data (and its format) as immutable. The code you write should move the raw data through a pipeline to your final analysis. You shouldn't have to run all of the steps every time you want to make a new figure (see [Analysis is a DAG](http://drivendata.github.io/cookiecutter-data-science/#analysis-is-a-dag)), but anyone should be able to reproduce the final products with only the code in `src` and the data in `data/raw`.

Also, if data is immutable, it doesn't need source control in the same way that code does. Therefore, ***by default, the data folder is included in the `.gitignore` file.*** If you have a small amount of data that rarely changes, you may want to include the data in the repository. Github currently warns if files are over 50MB and rejects files over 100MB. Some other options for storing/syncing large data include [AWS S3](https://aws.amazon.com/s3/) with a syncing tool (e.g., [`s3cmd`](http://s3tools.org/s3cmd)), [Git Large File Storage](https://git-lfs.github.com/), [Git Annex](https://git-annex.branchable.com/), and [dat](http://dat-data.com/). Currently by default, we ask for an S3 bucket and use [AWS CLI](http://docs.aws.amazon.com/cli/latest/reference/s3/index.html) to sync data in the `data` folder with the server.

### Notebooks Are for Exploration and Communication

Notebook packages like the [Jupyter notebook](http://jupyter.org/), [Beaker notebook](http://beakernotebook.com/), [Zeppelin](http://zeppelin-project.org/), and other literate programming tools are very effective for exploratory data analysis. However, these tools can be less effective for reproducing an analysis. When we use notebooks in our work, we often subdivide the `notebooks` folder. For example, `notebooks/exploratory` contains initial explorations, whereas `notebooks/reports` is more polished work that can be exported as html to the `reports` directory.

Since notebooks are challenging objects for source control (e.g., diffs of the `json` are often not human-readable and merging is near impossible), we recommended not collaborating directly with others on Jupyter notebooks. There are two steps we recommend for using notebooks effectively:

1. Follow a naming convention that shows the owner and the order the analysis was done in. We use the format `<step>-<ghuser>-<description>.ipynb` (e.g., `0.3-bull-visualize-distributions.ipynb`).
2. Refactor the good parts. Don't write code to do the same task in multiple notebooks. If it's a data preprocessing task, put it in the pipeline at `src/data/make_dataset.py` and load data from `data/interim`. If it's useful utility code, refactor it to `src`.

Now by default we turn the project into a Python package (see the `setup.py` file). You can import your code and use it in notebooks with a cell like the following:

```
# OPTIONAL: Load the "autoreload" extension so that code can change
%load_ext autoreload

# OPTIONAL: always reload modules so that as you change code in src, it gets loaded
%autoreload 2

from src.data import make_dataset
```

### Analysis is a DAG

Often in an analysis you have long-running steps that preprocess data or train models. If these steps have been run already (and you have stored the output somewhere like the `data/interim` directory), you don't want to wait to rerun them every time. We prefer [`make`](https://www.gnu.org/software/make/) for managing steps that depend on each other, especially the long-running ones. Make is a common tool on Unix-based platforms (and is available for Windows). Following the [`make` documentation](https://www.gnu.org/software/make/), [Makefile conventions](https://www.gnu.org/prep/standards/html_node/Makefile-Conventions.html#Makefile-Conventions), and [portability guide](http://www.gnu.org/savannah-checkouts/gnu/autoconf/manual/autoconf-2.69/html_node/Portable-Make.html#Portable-Make) will help ensure your Makefiles work effectively across systems. Here are [some](http://zmjones.com/make/) [examples](http://blog.kaggle.com/2012/10/15/make-for-data-scientists/) to [get started](https://web.archive.org/web/20150206054212/http://www.bioinformaticszen.com/post/decomplected-workflows-makefiles/). A number of data folks use `make` as their tool of choice, including [Mike Bostock](https://bost.ocks.org/mike/make/).

There are other tools for managing DAGs that are written in Python instead of a DSL (e.g., [Paver](http://paver.github.io/paver/#), [Luigi](http://luigi.readthedocs.org/en/stable/index.html), [Airflow](https://airflow.apache.org/index.html), [Snakemake](https://snakemake.readthedocs.io/en/stable/), [Ruffus](http://www.ruffus.org.uk/), or [Joblib](https://pythonhosted.org/joblib/memory.html)). Feel free to use these if they are more appropriate for your analysis.

### Build from the Environment up

The first step in reproducing an analysis is always reproducing the computational environment it was run in. You need the same tools, the same libraries, and the same versions to make everything play nicely together.

One effective approach to this is use [virtualenv](https://virtualenv.pypa.io/en/latest/) (we recommend [virtualenvwrapper](https://virtualenvwrapper.readthedocs.org/en/latest/) for managing virtualenvs). By listing all of your requirements in the repository (we include a `requirements.txt` file) you can easily track the packages needed to recreate the analysis. Here is a good workflow:

1. Run `mkvirtualenv` when creating a new project
2. `pip install` the packages that your analysis needs
3. Run `pip freeze > requirements.txt` to pin the exact package versions used to recreate the analysis
4. If you find you need to install another package, run `pip freeze > requirements.txt` again and commit the changes to version control.

If you have more complex requirements for recreating your environment, consider a virtual machine based approach such as [Docker](https://www.docker.com/) or [Vagrant](https://www.vagrantup.com/). Both of these tools use text-based formats (Dockerfile and Vagrantfile respectively) you can easily add to source control to describe how to create a virtual machine with the requirements you need.

### Keep Secrets and Configuration out of Version Control

You *really* don't want to leak your AWS secret key or Postgres username and password on Github. Enough said — see the [Twelve Factor App](http://12factor.net/config) principles on this point. Here's one way to do this:

#### Store Your Secrets and Config Variables in a Special File

Create a `.env` file in the project root folder. Thanks to the `.gitignore`, this file should never get committed into the version control repository. Here's an example:

```
# example .env file
DATABASE_URL=postgres://username:password@localhost:5432/dbname
AWS_ACCESS_KEY=myaccesskey
AWS_SECRET_ACCESS_KEY=mysecretkey
OTHER_VARIABLE=something
```

#### Use a Package to Load These Variables Automatically

If you look at the stub script in `src/data/make_dataset.py`, it uses a package called [python-dotenv](https://github.com/theskumar/python-dotenv) to load up all the entries in this file as environment variables so they are accessible with `os.environ.get`. Here's an example snippet adapted from the `python-dotenv` documentation:

```
# src/data/dotenv_example.py
import os
from dotenv import load_dotenv, find_dotenv

# find .env automagically by walking up directories until it's found
dotenv_path = find_dotenv()

# load up the entries as environment variables
load_dotenv(dotenv_path)

database_url = os.environ.get("DATABASE_URL")
other_variable = os.environ.get("OTHER_VARIABLE")
```

#### AWS CLI Configuration

When using Amazon S3 to store data, a simple method of managing AWS access is to set your access keys to environment variables. However, managing multiple sets of keys on a single machine (e.g. when working on multiple projects) it is best to use a [credentials file](https://docs.aws.amazon.com/cli/latest/userguide/cli-config-files.html), typically located in `~/.aws/credentials`. A typical file might look like:

```
[default]
aws_access_key_id=myaccesskey
aws_secret_access_key=mysecretkey

[another_project]
aws_access_key_id=myprojectaccesskey
aws_secret_access_key=myprojectsecretkey
```

You can add the profile name when initialising a project; assuming no applicable environment variables are set, the profile credentials will be used be default.

### Be Conservative in Changing the Default Folder Structure

To keep this structure broadly applicable for many different kinds of projects, we think the best approach is to be liberal in changing the folders around for *your* project, but be conservative in changing the default structure for *all* projects.

We've created a folder-layout label specifically for issues proposing to add, subtract, rename, or move folders around. More generally, we've also created a needs-discussion label for issues that should have some careful discussion and broad support before being implemented.

## Contributing

The Cookiecutter Data Science project is opinionated, but not afraid to be wrong. Best practices change, tools evolve, and lessons are learned. **The goal of this project is to make it easier to start, structure, and share an analysis.** [Pull requests](https://github.com/drivendata/cookiecutter-data-science/pulls) and [filing issues](https://github.com/drivendata/cookiecutter-data-science/issues) is encouraged. We'd love to hear what works for you, and what doesn't.

If you use the Cookiecutter Data Science project, link back to this page or [give us a holler](https://twitter.com/drivendataorg) and [let us know](mailto:info@drivendata.org)!

Project structure and reproducibility is talked about more in the R research community. Here are some projects and blog posts if you're working in R that may help you out.

- [Project Template](http://projecttemplate.net/index.html) - An R data analysis template
- "[Designing projects](http://nicercode.github.io/blog/2013-04-05-projects/)" on Nice R Code
- "[My research workflow](http://www.carlboettiger.info/2012/05/06/research-workflow.html)" on Carlboettiger.info
- "[A Quick Guide to Organizing Computational Biology Projects](http://journals.plos.org/ploscompbiol/article?id=10.1371/journal.pcbi.1000424)" in PLOS Computational Biology

Finally, a huge thanks to the [Cookiecutter](https://cookiecutter.readthedocs.org/en/latest/) project ([github](https://github.com/audreyr/cookiecutter)), which is helping us all spend less time thinking about and writing boilerplate and more time getting things done.

---

## Related Knowledge

> [!toc] `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[cookiecutter_data_science_automated_project_structure_and_files#Related Knowledge\|PKM]] | [[cookiecutter_data_science_automated_project_structure_and_files#Related Library Content\|Library]] | [[cookiecutter_data_science_automated_project_structure_and_files#Related Tasks and Events\|Tasks]] | [[cookiecutter_data_science_automated_project_structure_and_files#Related Directory\|Directory]] |
> |:--------: |:--------: |:--------: |:--------: |

> [!pkm] PKM Notes
>
> - `BUTTON[button-pkm-question-table]`|`BUTTON[button-pkm-evidence-table]`|`BUTTON[button-pkm-steps-table]`|`BUTTON[button-pkm-conclusion-table]`
> - `BUTTON[button-pkm-idea-table]`|`BUTTON[button-pkm-summary-table]`|`BUTTON[button-pkm-quote-table]`
> - `BUTTON[button-pkm-concept-table]`|`BUTTON[button-pkm-definition-table]`

<!-- Adjust replace lines -->

```button
name Related PKM Files
type append template
action 100_70_dvmd_related_pkm_sect
replace [1, 2]
color purple
```

### Outgoing PKM Links

<!-- Link related pkm files here -->

- [[data_analysis|Data Analysis]]
- [[python|Python]]

### Knowledge Tree

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
    default(((x) => {
      "category": "🏘️Category",
      "branch": "🪑Branch",
      "field": "🚪Field",
      "subject": "🗝️Subject",
      "topic": "🧱Topic"
    }[x])(file.frontmatter.type), "🔩Subtopic")
    AS Type,
    file.frontmatter.about AS Description,
    default(((x) => {
      "branch": file.frontmatter.category,
      "field": flat(list(file.frontmatter.category, file.frontmatter.branch)),
      "subject": flat(list(file.frontmatter.category, file.frontmatter.branch, file.frontmatter.field)),
      "topic": flat(list(file.frontmatter.category, file.frontmatter.branch, file.frontmatter.field, file.frontmatter.subject)),
      "subtopic": flat(list(file.frontmatter.category, file.frontmatter.branch, file.frontmatter.field, file.frontmatter.subject, file.frontmatter.topic))
    }[x])(file.frontmatter.type), "")
    AS Context
FROM
    "70_pkm"
WHERE
    file.name != this.file.name
	  AND contains(file.frontmatter.file_class, "pkm")
	  AND contains(file.frontmatter.file_class, "tree")
	  AND (contains(file.outlinks, this.file.link)
	  OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
      "category": 1,
      "branch": 2,
      "field": 3,
      "subject": 4,
      "topic": 5
    }[x])(file.frontmatter.type), 6),
    file.frontmatter.title ASC
```

### Permanent

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
	default(((x) => {
      "question": "❔Question",
      "evidence": "⚖️Evidence",
      "step": "🪜Step",
      "conclusion": "🎱Conclusion",

      "theorem": "🧮Theorem",

      "proof": "📃Proof",
      "quote": "⏺️Quote",
      "idea": "💭Idea",
      "summary": "📝Summary",
      "concept": "🎞️Concept"
    }[x])(file.frontmatter.type), "🪟Definition")
    AS Type,
	default(((x) => {
      "review": "📥Review",
      "clarify": "🌱Clarify",
      "develop": "🪴Develop",
      "permanent": "🌳Permanent"
    }[x])(file.frontmatter.status), "🗄️Resource")
    AS Status,
	  choice(!contains(
    ["evidence", "step", "conclusion", "summary"],
    file.frontmatter.type),
      filter(split(file.frontmatter.about, "\n"), (x) => regextest("\w", x)),
      file.frontmatter.about
    ) AS Content,
	  file.etags AS Tags
FROM
    "70_pkm"
WHERE
    file.name != this.file.name
	  AND contains(file.frontmatter.file_class, "pkm")
	  AND contains(file.frontmatter.type, "permanent")
	  AND (contains(file.outlinks, this.file.link)
	  OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
      "question": 1,
      "evidence": 2,
      "step": 3,
      "conclusion": 4,
      "theorem": 5,
      "proof": 6,
      "quote": 7,
      "idea": 8,
      "summary": 9,
      "concept": 10
    }[x])(file.frontmatter.type), 11),
	file.frontmatter.title ASC
```

### Literature

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
	default(((x) => {
      "question": "❔Question",
      "evidence": "⚖️Evidence",
      "step": "🪜Step",
      "conclusion": "🎱Conclusion",

      "theorem": "🧮Theorem",

      "proof": "📃Proof",
      "quote": "⏺️Quote",
      "idea": "💭Idea",
      "summary": "📝Summary",
      "concept": "🎞️Concept"
    }[x])(file.frontmatter.type), "🪟Definition")
    AS Type,
	default(((x) => {
      "review": "📥Review",
      "clarify": "🌱Clarify",
      "develop": "🪴Develop",
      "permanent": "🌳Permanent"
    }[x])(file.frontmatter.status), "🗄️Resource")
    AS Status,
	choice(!contains(
    ["evidence", "step", "conclusion", "summary"],
    file.frontmatter.type),
      filter(split(file.frontmatter.about, "\n"), (x) => regextest("\w", x)),
      file.frontmatter.about
    ) AS Content,
	file.etags AS Tags
FROM
    "70_pkm"
WHERE
    file.name != this.file.name
	  AND contains(file.frontmatter.file_class, "pkm")
	  AND contains(file.frontmatter.type, "literature")
	  AND (contains(file.outlinks, this.file.link)
	  OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
      "question": 1,
      "evidence": 2,
      "step": 3,
      "conclusion": 4,
      "theorem": 5,
      "proof": 6,
      "quote": 7,
      "idea": 8,
      "summary": 9,
      "concept": 10
    }[x])(file.frontmatter.type), 11),
	file.frontmatter.title ASC
```

### Fleeting

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
	default(((x) => {
      "question": "❔Question",
      "evidence": "⚖️Evidence",
      "step": "🪜Step",
      "conclusion": "🎱Conclusion",

      "theorem": "🧮Theorem",

      "proof": "📃Proof",
      "quote": "⏺️Quote",
      "idea": "💭Idea",
      "summary": "📝Summary",
      "concept": "🎞️Concept"
    }[x])(file.frontmatter.type), "🪟Definition")
    AS Type,
	default(((x) => {
      "review": "📥Review",
      "clarify": "🌱Clarify",
      "develop": "🪴Develop",
      "permanent": "🌳Permanent"
    }[x])(file.frontmatter.status), "🗄️Resource")
    AS Status,
	  choice(!contains(
    ["evidence", "step", "conclusion", "summary"],
    file.frontmatter.type),
      filter(split(file.frontmatter.about, "\n"), (x) => regextest("\w", x)),
      file.frontmatter.about
    ) AS Content,
	  file.etags AS Tags
FROM
    "70_pkm"
WHERE
    file.name != this.file.name
	  AND contains(file.frontmatter.file_class, "pkm")
	  AND contains(file.frontmatter.type, "fleeting")
	  AND (contains(file.outlinks, this.file.link)
	  OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
      "question": 1,
      "evidence": 2,
      "step": 3,
      "conclusion": 4,
      "theorem": 5,
      "proof": 6,
      "quote": 7,
      "idea": 8,
      "summary": 9,
      "concept": 10
    }[x])(file.frontmatter.type), 11),
	file.frontmatter.title ASC
```

### Info

```dataview
TABLE WITHOUT ID
    link(file.link, file.frontmatter.aliases[0]) AS Title,
	default(((x) => {
      "question": "❔Question",
      "evidence": "⚖️Evidence",
      "step": "🪜Step",
      "conclusion": "🎱Conclusion",

      "theorem": "🧮Theorem",

      "proof": "📃Proof",
      "quote": "⏺️Quote",
      "idea": "💭Idea",
      "summary": "📝Summary",
      "concept": "🎞️Concept"
    }[x])(file.frontmatter.type), "🪟Definition")
    AS Type,
	default(((x) => {
      "review": "📥Review",
      "clarify": "🌱Clarify",
      "develop": "🪴Develop",
      "permanent": "🌳Permanent"
    }[x])(file.frontmatter.status), "🗄️Resource")
    AS Status,
	  choice(!contains(
    ["evidence", "step", "conclusion", "summary"],
    file.frontmatter.type),
      filter(split(file.frontmatter.about, "\n"), (x) => regextest("\w", x)),
      file.frontmatter.about
    ) AS Content,
	  file.etags AS Tags
FROM
    "70_pkm"
WHERE
    file.name != this.file.name
	  AND contains(file.frontmatter.file_class, "pkm")
	  AND contains(file.frontmatter.type, "info")
	  AND (contains(file.outlinks, this.file.link)
	  OR contains(file.inlinks, this.file.link))
SORT
    default(((x) => {
      "question": 1,
      "evidence": 2,
      "step": 3,
      "conclusion": 4,
      "theorem": 5,
      "proof": 6,
      "quote": 7,
      "idea": 8,
      "summary": 9,
      "concept": 10
    }[x])(file.frontmatter.type), 11),
	file.frontmatter.title ASC
```

---

## Related Library Content

> [!toc] `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[cookiecutter_data_science_automated_project_structure_and_files#Related Knowledge\|PKM]] | [[cookiecutter_data_science_automated_project_structure_and_files#Related Library Content\|Library]] | [[cookiecutter_data_science_automated_project_structure_and_files#Related Tasks and Events\|Tasks]] | [[cookiecutter_data_science_automated_project_structure_and_files#Related Directory\|Directory]] |
> |:--------: |:--------: |:--------: |:--------: |

<!-- Adjust replace lines -->

```meta-bind-button
label: 🏫Related Library Content
tooltip: "Replace the library section with MD table of linked files and a filtered DataView table"
class: mb_button_green
hidden: false
style: default
actions:
  - type: replaceInNote
    fromLine: 1
    toLine: 2
    replacement: 00_system/07_templates/100_60_dvmd_related_lib_sect.md
    templater: true
```

### Outgoing Library Links

<!-- Link related library files here -->

### Library Content

```dataview
TABLE WITHOUT ID
	link(file.link, file.frontmatter.aliases[0]) AS Title,
	choice(length(choice(contains(file.frontmatter.type, "course"), file.frontmatter.lecturer, file.frontmatter.author)) < 2,
      choice(contains(file.frontmatter.type, "course"), file.frontmatter.lecturer[0], file.frontmatter.author[0]),
      flat(choice(contains(file.frontmatter.type, "course"), file.frontmatter.lecturer, file.frontmatter.author)))
    AS Creator,
	choice(contains(file.frontmatter.type, "book"),
		file.frontmatter.year_published,
		file.frontmatter.date_published)
	AS "Date Published",
	default(((x) => {
      "book": "📚Book",
      "book_chapter": "📑Book Chapter",
      "course": "🧑‍🏫Course",
      "course_lecture": "🧑‍🎓Course Lecture",
      "journal": "📜️Journal",
      "report": "📈Report",
      "news": "🗞️News",
      "magazine": "📰️Magazine",
      "webpage": "🌐Webpage",
      "blog": "💻Blog",
      "video": "🎥️Video",
      "youtube": "▶YouTube",
      "documentary": "🖼️Documentary",
      "audio": "🔉Audio",
      "podcast": "🎧️Podcast"
    }[x])(file.frontmatter.type), "📃Documentation")
	AS Type,
	default(((x) => {
      "undetermined": "❓Undetermined",
      "to_do": "🔜To do",
      "in_progress": "👟In progress",
      "done": "✔️Done",
      "resource": "🗃️Resource",
      "schedule": "📅Schedule"
    }[x])(file.frontmatter.status), "🤌On hold")
    AS Status,
	file.etags AS Tags
FROM
	"60_library"
	OR "90_inbox"
WHERE
	file.name != this.file.name
	AND (contains(file.outlinks, this.file.link)
	OR contains(file.inlinks, this.file.link))
	AND contains(file.frontmatter.file_class, "lib")
	AND contains(file.frontmatter.file_class, "")
SORT
	file.frontmatter.type,
	file.frontmatter.title ASC
```

---

## Related Tasks and Events

> [!toc] `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[cookiecutter_data_science_automated_project_structure_and_files#Related Knowledge\|PKM]] | [[cookiecutter_data_science_automated_project_structure_and_files#Related Library Content\|Library]] | [[cookiecutter_data_science_automated_project_structure_and_files#Related Tasks and Events\|Tasks]] | [[cookiecutter_data_science_automated_project_structure_and_files#Related Directory\|Directory]] |
> |:--------: |:--------: |:--------: |:--------: |

> [!task] Tasks and Events
>
> `BUTTON[button-project-task-table]`|`BUTTON[button-parent-task-table]`|`BUTTON[button-action-item-task-table]`|`BUTTON[button-meeting-task-table]`

<!-- Adjust replace lines -->

```button
name ✅Related Tasks and Events
type append template
action 100_40_dvmd_related_task_sect
replace [1, 2]
color blue
```

### Outgoing Task and Events Links

<!-- Link related tasks and events here -->

### Projects

```dataview
TABLE WITHOUT ID
    link(file.name, file.frontmatter.aliases[0]) AS Project,
    default(((x) => {
      "done": "✔️Done",
      "in_progress": "👟In progress",
      "to_do": "🔜To do",
      "schedule": "📅Schedule",
      "on_hold": "🤌On hold",
      "applied": "📨Applied💼",
      "offer": "📝Job Offer💼",
      "rejected": "🚫Rejected💼"
    }[x])(file.frontmatter.status), "❌Discarded")
	AS Status,
    choice((regextest("\d", file.frontmatter.task_start) AND regextest("\d", file.frontmatter.task_end)),
		(dateformat(date(regexreplace(file.frontmatter.task_start, "[^\d-]", "")), "yy-MM-dd") + " → " + dateformat(date(regexreplace(file.frontmatter.task_end, "[^\d-]", "")), "yy-MM-dd")),
		choice(regextest("\d", file.frontmatter.task_start),
			(dateformat(date(regexreplace(file.frontmatter.task_start, "[^\d-]", "")), "yy-MM-dd") + " → Present"),
			"null"))
	AS Dates,
    choice(file.frontmatter.context = "habit_ritual", "Habits and Rituals",
    upper(substring(file.frontmatter.context, 0, 1)) + substring(file.frontmatter.context, 1))
    AS Context,
    Objective AS Objective,
    list(("**Outcome**: " + Outcome), ("**Feeling**: " + Feeling)) AS Result
FROM
    "41_personal"
    OR "42_education"
    OR "43_professional"
    OR "44_work"
    OR "45_habit_ritual"
WHERE
    file.name != this.file.name
    AND contains(file.frontmatter.file_class, "task")
    AND contains(file.frontmatter.file_class, "project")
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
    AND !contains(file.path, this.file.folder)
SORT
    file.frontmatter.title ASC
```

### Parent Tasks

```dataview
TABLE WITHOUT ID
    link(file.name, file.frontmatter.aliases[0]) AS "Parent Task",
    default(((x) => {
      "done": "✔️Done",
      "in_progress": "👟In progress",
      "to_do": "🔜To do",
      "schedule": "📅Schedule",
      "on_hold": "🤌On hold",
      "applied": "📨Applied💼",
      "offer": "📝Job Offer💼",
      "rejected": "🚫Rejected💼"
    }[x])(file.frontmatter.status), "❌Discarded")
	AS Status,
    choice((regextest("\d", file.frontmatter.task_start) AND regextest("\d", file.frontmatter.task_end)),
		(dateformat(date(regexreplace(file.frontmatter.task_start, "[^\d-]", "")), "yy-MM-dd") + " → " + dateformat(date(regexreplace(file.frontmatter.task_end, "[^\d-]", "")), "yy-MM-dd")),
		choice(regextest("\d", file.frontmatter.task_start),
			(dateformat(date(regexreplace(file.frontmatter.task_start, "[^\d-]", "")), "yy-MM-dd") + " → Present"),
			"null"))
	AS Dates,
    choice(length(file.frontmatter.project) < 2, file.frontmatter.project[0], flat(file.frontmatter.project)) AS Project,
    Objective AS Objective,
    list(("**Outcome**: " + Outcome), ("**Feeling**: " + Feeling)) AS Result
FROM
    "41_personal"
    OR "42_education"
    OR "43_professional"
    OR "44_work"
    OR "45_habit_ritual"
WHERE
    file.name != this.file.name
    AND contains(file.frontmatter.file_class, "task")
    AND contains(file.frontmatter.file_class, "parent")
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
    AND !contains(file.path, this.file.folder)
SORT
    file.frontmatter.task_start,
    file.frontmatter.title ASC
```

### Child Tasks

```dataview
TABLE WITHOUT ID
    link(T.link, regexreplace(T.text, "#task\s(.+)_(action_item|meeting|phone_call|interview|appointment|event|gathering|hangout|habit|morning_ritual|workday_startup_ritual|workday_shutdown_ritual|evening_ritual).+", "$1")) AS Task,
    choice(contains(T.text, "_act"), "🔨Task",
    choice(contains(T.text, "_meet"), "🤝Meeting",
    choice(contains(T.text, "_phone"), "📞Call",
    choice(contains(T.text, "_int"), "💼Interview",
    choice(contains(T.text, "_app"), "⚕️Appointment",
    choice(contains(T.text, "_event"), "🎊Event",
    choice(contains(T.text, "_gath"), "✉️Gathering",
    choice(contains(T.text, "_hang"), "🍻Hangout",
    choice(contains(T.text, "_habit"), "🤖Habit",
    choice(contains(T.text, "_morn"), "🍵Rit.",
    choice(contains(T.text, "day_start"), "🌇Rit.",
    choice(contains(T.text, "day_shut"), "🌆Rit.", "🛌Rit."))))))))))))
    AS Type,
    choice((T.status != "-"),
        (choice((T.status = "x"), "✔️Done", "🔜To do")),
        "❌Discard")
    AS Status,
    choice(T.status = "x", dateformat(T.completion, "yy-MM-dd"), dateformat(T.due, "yy-MM-dd"))
    AS Date,
    choice(length(file.frontmatter.project) < 2, file.frontmatter.project[0], flat(file.frontmatter.project)) AS Project
FROM
    "41_personal"
    OR "42_education"
    OR "43_professional"
    OR "44_work"
    OR "45_habit_ritual"
FLATTEN
    file.tasks AS T
WHERE
    file.name != this.file.name
    AND contains(file.frontmatter.file_class, "task")
    AND contains(file.frontmatter.file_class, "child")
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
    AND !contains(file.path, this.file.folder)
    AND regextest("(#task)", T.text)
SORT
    T.due,
    T.time_start ASC
```

---

## Related Directory

> [!toc] `dv: link(this.file.name + "#" + this.file.frontmatter.aliases[0], "Contents")`
>
> | [[cookiecutter_data_science_automated_project_structure_and_files#Related Knowledge\|PKM]] | [[cookiecutter_data_science_automated_project_structure_and_files#Related Library Content\|Library]] | [[cookiecutter_data_science_automated_project_structure_and_files#Related Tasks and Events\|Tasks]] | [[cookiecutter_data_science_automated_project_structure_and_files#Related Directory\|Directory]] |
> |:--------: |:--------: |:--------: |:--------: |

<!-- Adjust replace lines -->

```meta-bind-button
label: 📇Related Directory Files
tooltip: "Replace the directory section with MD table of linked files and a filtered DataView table"
class: mb_button_pink
style: default
hidden: false
actions:
  - type: replaceInNote
    fromLine: 1
    toLine: 2
    replacement: 00_system/07_templates/100_50_dvmd_related_dir_sect.md
    templater: true
```

### Outgoing Contact Links

<!-- Link related contacts here -->

### Contacts

```dataview
TABLE WITHOUT ID
    link(file.name, file.frontmatter.aliases[0]) AS Name,
    file.frontmatter.job_title AS "Job Title",
    file.frontmatter.organization AS Organization,
    file.etags AS Tags
FROM
    "51_contacts"
WHERE
    file.name != this.file.name
    AND contains(file.frontmatter.file_class, "dir")
    AND contains(file.frontmatter.file_class, "contact")
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
SORT
    file.frontmatter.title ASC
```

### Outgoing Organization Links

<!-- Link related organizations here -->

### Organizations

```dataview
TABLE WITHOUT ID
    link(file.name, file.frontmatter.aliases[0]) AS Name,
    elink(file.frontmatter.url, "Website") AS Website,
    elink(file.frontmatter.linkedin_url, "LinkedIn") AS LinkedIn,
    file.frontmatter.about AS About,
    file.etags AS Tags
FROM
    "52_organizations"
WHERE
    file.name != this.file.name
    AND contains(file.frontmatter.file_class, "dir")
    AND contains(file.frontmatter.file_class, "organization")
    AND (contains(file.outlinks, this.file.link)
    OR contains(file.inlinks, this.file.link))
SORT
    file.frontmatter.title ASC
```

---
