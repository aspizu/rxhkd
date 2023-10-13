from __future__ import annotations
from sys import stdin
from typing import Literal
from msgspec import json
from msgspec.structs import Struct
from rich import print

SHIFT = 1
CAPSLOCK = 2
CTRL = 4
ALT = 8
NUMLOCK = 16
MOD3 = 32
SUPER = 64
MOD5 = 128


def modnames(mod: int) -> list[str]:
    mods = []
    if mod & SHIFT:
        mods.append("shift")
    if mod & CAPSLOCK:
        mods.append("caps-lock")
    if mod & CTRL:
        mods.append("ctrl")
    return mods


class Chord(Struct):
    modifiers: int
    key: str

    def __str__(self):
        return (
            '   <div class="Chord">'
            + "".join(
                (
                    f'    <span class="Chord__key">{i}</span>'
                    for i in modnames(self.modifiers)
                )
            )
            + f'    <span class="Chord__key">{self.key}</span>'
            + " </div>"
        )


class EnterMode(Struct):
    binds: list[Bind]

    def __str__(self):
        return f'<div class="EnterMode">{"".join(map(str,self.binds))}</div>'


class ActionEnterMode(Struct):
    EnterMode: EnterMode

    def __str__(self):
        return str(self.EnterMode)


class Bind(Struct):
    chord: Chord
    output: str | None
    action: ActionEnterMode | Literal["None"]

    def __str__(self):
        return (
            f'<div class="Bind">'
            f"    {self.chord}"
            f'    <span class="Bind__output">'
            f"        <pre>"
            f"{self.output and self.output.strip().replace(chr(10), '')}"
            f"</pre>"
            f"    </span>"
            f"</div>"
        )


binds = json.decode(stdin.read(), type=list[Bind])
print("<!DOCTYPE html>")
print("<head>")
print('<link rel="stylesheet" href="style.css" />')
print("</head>")
print("<body>")
print("".join(map(str, binds)))
print("</body>")
print("<html>")
