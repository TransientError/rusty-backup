This is a rust application meant to help you maintain backups as a cron or anacron job.

You can configure what to backup and where to backup to in the configurations. For example looking at this from [backup_sample](https://github.com/TransientError/rusty-backup/blob/master/backup_sample.json)

```
{
    "archives": [
        {
            "name": "test",
            "content": "echo Some stuff"
        }
    ],
    "backups": [
        {
            "name": "hi",
            "custom": "echo"
        }
    ]
}
```

The archives are a list of objects where the name specifies the name of the thing you want to save and the content is a unix command that prints what you want to save to std out (e.g. cargo --list or brew list etc)
The backups list are commands that then take result of generating all the archives and then uploads each archive onto each backup. There is a custom backup type for github snippets, which is what I use most.
