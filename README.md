# EML-REPLICATOR 🤖

This tool read all the EML (RFC822 / RFC2822) in a directory and copy them in a IMAP folder on the mailbox of your choice.

Usage:

```
eml-replicator 0.1.2
Maël Naccache Tüfekçi
A tool that read EML files and copy them to a IMAP mailbox.

USAGE:
    eml-replicator.exe [FLAGS] [OPTIONS] <IMAP_SERVER> <DIR>

FLAGS:
    -h, --help                 Prints help information
        --random-message-id    Randomize the Message-ID in the emls before sending them.
    -r, --recursive            Goes through the directory recursively to find EML files.
    -s, --follow-symlink       Follow symlink when crawling the directory recursively.
    -V, --version              Prints version information

OPTIONS:
    -f, --folder <FOLDER>            IMAP Folder in which to put the EMLs. [default: INBOX]
    -l, --login <LOGIN>              login of the mailbox. [default: ]
    -p, --password <PASSWORD>        password of the mailbox. [default: ]
        --port <IMAP_SERVER_PORT>    Port to connect to the imap server. [default: 993]

ARGS:
    <IMAP_SERVER>    IMAP server to connect to.
    <DIR>            Directory in which to get the EML files. [default: .]
```
