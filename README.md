# EML-REPLICATOR 🤖

This tool read all the EML (RFC822 / RFC2822) in a directory and copy them in a IMAP folder on the mailbox of your choice.

Usage:

```
eml-replicator 1.1
Maël Naccache Tüfekçi
A tool that read EML files and copy them to a IMAP mailbox.

USAGE:
    eml-replicator.exe [FLAGS] [OPTIONS] <IMAP_SERVER> <DIR>

FLAGS:
    -h, --help                 Prints help information
    -r, --recursive            Goes through the directory recursively to find EML files.
    -s, --follow-symlink       Follow symlink when crawling the directory recursively.
    -V, --version              Prints version information

OPTIONS:
        --port <IMAP_SERVER_PORT>    Port to connect to the imap server. [default: 993]

ARGS:
    <IMAP_SERVER>    IMAP server to connect to.
    <DIR>            Directory in which to get the EML files. [default: .]
```
