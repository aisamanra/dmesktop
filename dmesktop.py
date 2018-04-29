#!/usr/bin/env python3

import os
import re
import subprocess
import sys
from typing import List, Mapping, NamedTuple, Union

# compiled regexes for faux-parsing INI files
KV = re.compile('([A-Za-z0-9-]*(\[[A-Za-z@_]*\])?)=(.*)')
SECTION = re.compile('\[([A-Za-z -]*)\]')
METAVAR = re.compile('%.')
# default argv for running dmenu
DMENU_CMD = ['dmenu', '-i', '-l', '10']

# this is probably not right, in the long term!
XDG_APP_DIRS = [
    '/usr/share/applications',
    os.path.join(os.getenv('HOME'), '/.local/share/applications'),
]

class DesktopEntry(NamedTuple):
    '''This is our wrapper struct for .desktop files. Right now, we only
    bother caring about the name, the command to execute, the type,
    and the other associated actions. Eventually this could be expanded?
    '''
    name: str
    exec: str
    type: str
    actions: Mapping[str, 'DesktopEntry']

    @classmethod
    def from_map(cls, m: Mapping[str, Union[str, Mapping[str, str]]]) -> 'DesktopEntry':
        '''Constructor function to take the key-value map we create in reading
        the file and turn it into a DesktopEntry.
        '''
        actions = dict(((e['Name'], cls.from_map(e))
                       for e in m.get('ACTIONS', {}).values()))
        return cls(
            m['Name'],
            m['Exec'],
            m.get('Type', None),
            actions,
        )

    def command(self) -> str:
        '''Clean out the metavariables we don't care about in
        the provided Exec field
        '''
        return METAVAR.sub('', self.exec)

def main():
    ensure_dmenu()
    desktop_entries = get_all_entries()

    # let's only ask about Applications for now, at least
    all_choices = sorted([key for (key, e)
                          in desktop_entries.items()
                          if e.type == 'Application'])
    choice = dmenu_choose(all_choices)
    if choice in desktop_entries:
        try:
            entry = desktop_entries[choice]
            if not entry.actions:
                os.execvp('/bin/sh', ['sh', '-c', entry.command()])
            else:
                choice = dmenu_choose(entry.actions.keys())
                if choice in entry.actions:
                    entry = entry.actions[choice]
                    os.execvp('/bin/sh', ['sh', '-c', entry.command()])
        except Exception as e:
            # this should be more granular eventually!
            pass

def dmenu_choose(stuff: List[str]) -> str:
    '''Given a list of strings, we provide them to dmenu and
    return back which one the user chose (or an empty string,
    if the user chose nothing)
    '''
    choices = '\n'.join(stuff).encode('utf-8')
    dmenu = subprocess.Popen(
        DMENU_CMD,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE)
    # we probably shouldn't ignore stderr, but whatevs
    choice, _ = dmenu.communicate(choices)
    return choice.decode('utf-8').strip()

def get_all_entries() -> Mapping[str, DesktopEntry]:
    '''Walk the relevant XDG dirs and parse all the Desktop files we can
    find, returning them as a map from the Name of the desktop file to
    the actual contents
    '''
    desktop_entries = {}

    # walk all the app dirs
    for dir in XDG_APP_DIRS:
        for root, dirs, files in os.walk(dir):
            # for whatever .desktop files we find
            for f in files:
                if f.endswith('.desktop'):
                    # add 'em to our Name-indexed map of files
                    entry = parse_entry(os.path.join(root, f))
                    desktop_entries[entry.name] = entry

    return desktop_entries

def parse_entry(path: str) -> DesktopEntry:
    '''Read and parse the .desktop file at the provided path
    '''
    # the `entry` is the basic key-value bag for the whole file
    entry = {}
    # but `current_bag` points to the current section being parsed,
    # which may or may not be the current overall file
    current_bag = entry
    with open(path) as f:
        for line in f:
            # this will be non-None if it's of the form `Key=Value`
            match = KV.match(line)
            # this will be non-None if it's of the form `[Name]`
            sect = SECTION.match(line)
            if match:
                # if it's a key-value pair, add it to the current bag
                current_bag[match.group(1)] = match.group(3)
            elif sect and sect.group(1) != 'Desktop Entry':
                # if it's a section header, then we ask: is it the
                # Desktop Entry section, which is the main obligatory
                # chunk of a desktop file? If so, then we ignore this
                # chunk entirely. Otherwise: make sure we have an
                # ACTIONS field
                actions = entry.get('ACTIONS', {})
                # create a new key/value map for this new sub-entry
                current_bag = {}
                # add the new key/value map to the actions map
                actions[sect.group(1)] = current_bag
                # and make sure the ACTIONS key points to that map
                entry['ACTIONS'] = actions

    # wrap it in in our nice wrapper type, too!
    return DesktopEntry.from_map(entry)

def ensure_dmenu():
    '''Shells out to `which` to find out whether dmenu is installed. We
    won't do anything useful if it's not, after all!
    '''
    try:
        subprocess.check_output(['which', 'dmenu'])
    except subprocess.CalledProcessError:
        sys.stderr.write("Error: could not find `dmenu'\n")
        sys.exit(99)


if __name__ == '__main__':
    main()
