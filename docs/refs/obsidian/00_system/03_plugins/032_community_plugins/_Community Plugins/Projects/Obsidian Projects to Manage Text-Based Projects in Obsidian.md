---
title: Obsidian Projects_A Better Way to Manage Text-Based Projects in Obsidian
author: Prakash Joshi Pax
publisher: Medium
date_published: 2022-11-19
hostname: beingpax.medium.com
url: https://beingpax.medium.com/obsidian-projects-a-better-way-to-manage-text-based-projects-in-obsidian-18c2a991069c
date_created: 2023-03-05T00:00
tags: obsidian, obsidian/plugin, obsidian/projects
date_modified: 2023-09-05T19:17
---
# Obsidian Projects: A Better Way to Manage Text-Based Projects in Obsidian

## Get Rid of All Redundant Plugins

> [!Excerpt]
> Obsidian is great for writing but not that much for planning and organization. But the plugin we’re talking about today fills that gap in obsidian. The plugin is called obsidian projects. And it…

---

## Introduction

![[1RARHYW3kujqMK-XJxquTZw.webp]]

Obsidian is great for writing but not that much for planning and organization. But the plugin we’re talking about today fills that gap in obsidian.

The plugin is called obsidian projects. And it stands by its name. It helps you to manage your projects inside obsidian. It helps to visualize your notes for project management.

It seems to somehow replace other plugins like kanban view, Fantasy calendar, and database folder plugin. All of these views are integrated within this plugin. And guess what, you can switch between all of these views.

Let me walk you through the setup process of the plugin and how to start managing your projects right inside of obsidian.

### Installing the Plugin

Go to the community plugins section and search for **‘Projects’** by Marcus Olsson. Install and enable the plugin.

![[1lNvbF4B-Z0vacDlFfnaK5A.webp]]

### How Does This Plugin Work?

This plugin treats projects as a collection of notes. Once you have added your query for the notes, it collects data from the frontmatter and displays it in different views like table, calendar, gallery, and board.

## How to Create a New Project

Go to the command palette and search for ‘Projects’.

![[1BMmVM-HGw4bQqYBQ7ehLrg.webp]]

Click on create a new project. A new window will appear for the configuration of our project. Let’s explore the configuration

- **Use Dataview:** This plugin does queries in two ways. One is using the dataview plugin, other is using paths. If you enable the dataview query, the project will be read-only. Therefore, I will not use the dataview query here.
- **Include subfolders:** If you use path query, this option determines whether you include the subfolders of the path as project files or not.
- **Templates:** You can configure a particular template to be used for all your project notes.

![[1acpiw9XazZtWOGQ72Vo9gQ.webp]]

This plugin will collect data from the *frontmatter.* So, there are certain metadata properties that can be helpful to know while setting up obsidian projects.

- **Status:** This property can be used to define the status of the note. Whether it is finished, yet to do, or work in progress. This is helpful for organizing your notes in Board(kanban) view.
- **Priority:** This property can only include numerical value. It can be used to rank the project notes based on its importance.
- **Date:** Adding a date property to your note will help it display on the calendar.
- **Check:** As the name itself suggests, the check property can be used to show the status of whether it is done or not.
- **Cover:** This property is used to display images in the gallery view of your projects.

Based on all of these metadata properties, here’s the template I created.

```
---
Status: Up Next
Published: false
priority: 3
tags: youtube
cover: https://i.insider.com/624b335a82200b001943f793?width=1136&format=jpeg
due: 2023-03-16
---

```

I added a default cover image and templater function to add a due date 1 week from the date which the note is created(it can be changed too).

## Visualizing Our Projects

I created a new project and some notes for this project called ‘content creation’. This is the default table view of the plugin

![[1SHkAJAHXZypiVQ_Ky727Kw.webp]]

hiding unnecessary fields in the table view

### Board View

The boar view aka. kanban view is an excellent way to visualize the status of your project. Go to the new button and choose view from there to create a board view.

![[1cg5Fgsq1Ml2mJnKM2Rj2CQ.webp]]

The board groups notes in different columns. The textual field status is used for grouping notes & priority field is used to rank those notes by their importance.

You can create different columns in the board view by adding different status to the notes.

![[1rh0AKC2JLYVrftZ9jmEHOQ.webp]]

Board view in obsidian projects plugins

Currently, you can’t move notes between columns, but it's in the [roadmap.](https://github.com/users/marcusolsson/projects/4/views/14) Hopefully, we can see the feature soon.

### Calendar View

Calendar view uses the date property and the check(true/false) property.

![[1RjZPv_gnK8vN-hLEMtWdng.webp]]

Calendar view in projects plugin

You can also create a new note from the calendar view by double tapping the date. Even if you use a template, this action will override the date property in your note.

The feature to drag your notes to different dates is also on the [roadmap.](https://github.com/users/marcusolsson/projects/4/views/14)

### Gallery View

Gallery view lays out all of your notes in grid view. Since we have added a default image to all of our notes, here’s what the gallery view looks like:

![[11g3q6E9nLRD7cXqY1Ml9VA.webp]]

Gallery view in projects plugin

## Bottomline

[Obsidian projects](https://github.com/marcusolsson/obsidian-projects) by [Marcus olsson](https://marcus.se.net/) is a great plugin to help you manage your text-based projects. With this plugin, you can even replace other plugins like database folder, fantasy calendar, and kanban.
