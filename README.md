# scissors

## Usage

- Grab [the latest release](https://github.com/TicClick/scissors/releases/latest) for your platform.
- In [osu! account settings § OAuth](https://osu.ppy.sh/home/account/edit), add a new application (redirect URL may be any).
- Pass client ID and client secret to the app whenever necessary.
- To get help about a command:
  ```sh
  ./scissors <command> --help
  ```

## Commands

### `users`

Compare chunks like below against the data from osu! API and verify country codes and usernames.

```md
::{ flag=CA }:: [71cCl1ck](https://osu.ppy.sh/users/672931)
```

To check for missing flags, pass `--required`.
