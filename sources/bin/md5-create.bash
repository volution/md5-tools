#!/bin/bash

set -e -E -u -o pipefail -o noclobber -o noglob +o braceexpand || exit -- 1
trap 'printf -- "[ee] failed: %s\n" "${BASH_COMMAND}" >&2' ERR || exit -- 1

test "${#}" == 1

_source="${1}"

_source="$( exec -- readlink -e -- "${_source}" )"
_timestamp="$( exec -- date '+%Y-%m-%d-%H-%M-%S' )"

if test -e "${_source}/.md5" ; then
	_target="${_source}/.md5/${_timestamp}.md5"
	_target_skip="./.md5/${_timestamp}.md5"
else
	_target="${_source}/.--${_timestamp}.md5"
	_target_skip="./.--${_timestamp}.md5"
fi

test -d "${_source}"
test ! -e "${_target}"

cd -- "${_source}"

test ! -e "${_target}"
test ! -e "${_target}.tmp"
touch -- "${_target}.tmp"
test -f "${_target}.tmp"

find \
		. \
		-xdev \
		\( -type d -exec test -e {}/.md5.excluded \; -prune \) -o \
		\( \
			-type f \
			-not -path "${_target_skip}.tmp" \
			-print0 \
		\) \
| LC_ALL=C sort -z \
| xargs -r -0 -n 64 -- md5sum -b -- \
| tee -- "${_target}.tmp"

chmod 400 -- "${_target}.tmp"
mv -n -T -- "${_target}.tmp" "${_target}"

exit -- 0

