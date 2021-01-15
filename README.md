# EML-REPLICATOR

This tool read all the EML (RFC822 / RFC2822) in a directory and copy them in a IMAP folder on the mailbox of your choice.

Usage:

```
eml-replicator 1.0
Maël Naccache Tüfekçi
A tool that read EML files and copy them to a IMAP mailbox.

USAGE:
    eml-replicator.exe [OPTIONS] <IMAP_SERVER> <DIR>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -l, --login <LOGIN>              login of the mailbox. [default: ]
    -p, --password <PASSWORD>        password of the mailbox. [default: ]
    -s, --port <IMAP_SERVER_PORT>    Port to connect to the imap server. [default: 993]

ARGS:
    <IMAP_SERVER>    IMAP server to connect to.
    <DIR>            Directory in which to get the EML files. [default: .]
```
