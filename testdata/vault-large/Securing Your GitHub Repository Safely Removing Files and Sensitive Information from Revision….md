---
title: "Securing Your GitHub Repository: Safely Removing Files and Sensitive Information from Revision…"
source: "https://ishaileshmishra.medium.com/securing-your-github-repository-safely-removing-files-and-sensitive-information-from-revision-9d4127904f4f"
author:
  - "[[Shailesh Mishra]]"
published: 2024-02-02
created: 2025-05-12
description: "Securing Your GitHub Repository: Safely Removing Files and Sensitive Information from Revision History: Begin by installing Gitleaks on your local machine to scrutinize your repository for…"
tags:
  - "clippings"
---
![|614x407](https://miro.medium.com/v2/resize:fit:700/0*xw9UEdfxH97n0mUs)

Photo by Yancy Min on Unsplash

*Securing Your GitHub Repository: Safely Removing Files and Sensitive Information from Revision History:*

- ***Removing Crazy Big Files***
- ***Removing Passwords, Credentials & other Private data***

**Step 1: Install Gitleaks**

Begin by installing Gitleaks on your local machine to scrutinize your repository for inadvertent commits containing sensitive data. Follow the installation instructions outlined [here](https://github.com/gitleaks/gitleaks#getting-started).

*Additionally, consider integrating Gitleaks into your Continuous Integration (CI) pipeline to automatically check for leaks with every code change. Refer to* [*this link*](https://github.com/gitleaks/gitleaks#github-action) *for CI pipeline integration.*

To identify vulnerabilities, navigate to your project’s root folder and execute the following command:

```c
gitleaks detect -report-path gitleaks-report.json
```

A report named `gitleaks-report.json` will be generated in the root folder, providing a comprehensive overview of sensitive information detected in your repository like below.

**Report Example:**

```c
[
 {
  "Description": "Generic API Key",
  "StartLine": 57,
  "EndLine": 57,
  "StartColumn": 33,
  "EndColumn": 63,
  "Match": "API_KEY = \"8982c8ad610ff4ddc2\"",
  "Secret": "8982c8ad610ff4ddc2",
  "File": "src/java/com/shaileshmishra/io/Credentials.java",
  "SymlinkFile": "",
  "Commit": "030dfe48a35b3398c853c721b7eee56561b81c01",
  "Entropy": 3.5766177,
  "Author": "Shailesh Mishra",
  "Email": "shailesh@gmail.com",
  "Date": "2023-08-09T13:14:57Z",
  "Message": "run ci",
  "Tags": [],
  "RuleID": "generic-api-key",
  "Fingerprint": "030dfe48a35b3398c853c721b7eee56561b81c01:src/java/com/shaileshmishra/io/Credentials.java:generic-api-key:57" },
 {
  "Description": "Generic API Key",
  "StartLine": 58,
  "EndLine": 58,
  "StartColumn": 42,
  "EndColumn": 77,
  "Match": "TOKEN = \"9c1afdffa298f8708e3459e4\"",
  "Secret": "9c1afdffa298f8708e3459e4",
  "File": "src/java/com/shaileshmishra/io/Credentials.java",
  "SymlinkFile": "",
  "Commit": "030dfe48a35b3398c853c721b7eee56561b81c01",
  "Entropy": 3.7192945,
  "Author": "Shailesh Mishra",
  "Email": "shailesh@gmail.com",
  "Date": "2023-08-09T13:14:57Z",
  "Message": "run ci",
  "Tags": [],
  "RuleID": "generic-api-key",
  "Fingerprint": "030dfe48a35b3398c853c721b7eee56561b81c01:src/java/com/shaileshmishra/io/Credentials.java:generic-api-key:58" }
]
```

If the report reveals sensitive information in a file, such as `src/java/com/shaileshmishra/io/Credentials.java`, proceed to the next steps for secure removal.

to do that we have to follow few more steps

**Step 2: Utilize BFG Repo-Cleaner**

- Visit [this link](https://rtyley.github.io/bfg-repo-cleaner/) and download the Java JAR file indicated with a red border as depicted in the below image.
![](https://miro.medium.com/v2/resize:fit:640/format:webp/1*KYXN5CX8_wi5uIrSUtjApw.png)

*Note: Ensure that you have Java JDK installed on your machine.*

- Execute the following command in the terminal (replace the file name with the downloaded version):
```c
java -jar bfg-1.14.0.jar
```

Result shows like below once you run above.

```c
shaileshmishra@shaileshmishra-MacBook-Pro ~ % java -jar  /Users/shaileshmishra/Downloads/bfg-1.14.0.jar
bfg 1.14.0
Usage: bfg [options] [<repo>]

  -b, --strip-blobs-bigger-than <size>
                           strip blobs bigger than X (eg '128K', '1M', etc)
  -B, --strip-biggest-blobs NUM
                           strip the top NUM biggest blobs
  -bi, --strip-blobs-with-ids <blob-ids-file>
                           strip blobs with the specified Git object ids
  -D, --delete-files <glob>
                           delete files with the specified names (eg '*.class', '*.{txt,log}' - matches on file name, not path within repo)
  --delete-folders <glob>  delete folders with the specified names (eg '.svn', '*-tmp' - matches on folder name, not path within repo)
  --convert-to-git-lfs <value>
                           extract files with the specified names (eg '*.zip' or '*.mp4') into Git LFS
  -rt, --replace-text <expressions-file>
                           filter content of files, replacing matched text. Match expressions should be listed in the file, one expression per line - by default, each expression is treated as a literal, but 'regex:' & 'glob:' prefixes are supported, with '==>' to specify a replacement string other than the default of '***REMOVED***'.
  -fi, --filter-content-including <glob>
                           do file-content filtering on files that match the specified expression (eg '*.{txt,properties}')
  -fe, --filter-content-excluding <glob>
                           don't do file-content filtering on files that match the specified expression (eg '*.{xml,pdf}')
  -fs, --filter-content-size-threshold <size>
                           only do file-content filtering on files smaller than <size> (default is 1048576 bytes)
  -p, --protect-blobs-from <refs>
                           protect blobs that appear in the most recent versions of the specified refs (default is 'HEAD')
  --no-blob-protection     allow the BFG to modify even your *latest* commit. Not recommended: you should have already ensured your latest commit is clean.
  --private                treat this repo-rewrite as removing private data (for example: omit old commit ids from commit messages)
  --massive-non-file-objects-sized-up-to <size>
                           increase memory usage to handle over-size Commits, Tags, and Trees that are up to X in size (eg '10M')
  <repo>                   file path for Git repository to clean
```

**STEP: 4**

1. To delete sensitive files, execute the following command:
```c
bfg --delete-files Credentials.java
```

**Step 4: Confirm Deletion**

After performing the deletion operation, run the following command to confirm the changes:

```c
git reflog expire --expire=now --all && git gc --prune=now --aggressive
```

**Additional Resources:**

> For more detailed information and assistance, refer to the following links:

- [BFG Repo-Cleaner Documentation](https://rtyley.github.io/bfg-repo-cleaner/)
- [BFG Repo-Cleaner GitHub Repository](https://github.com/rtyley/bfg-repo-cleaner)

> Happy coding and maintaining a secure repository!

## More from Shailesh Mishra

## Recommended from Medium

[

See more recommendations

](https://medium.com/?source=post_page---read_next_recirc--9d4127904f4f---------------------------------------)
