# there must be a better way to do this but wev

all: ace.js theme-twilight.js ansi_up.js viz.js viz.render.js

ace.js:
	curl https://raw.githubusercontent.com/ajaxorg/ace-builds/d979a82c6198802cfb6df5be236379d29f116b2d/src-noconflict/ace.js > $@

theme-twilight.js:
	curl https://raw.githubusercontent.com/ajaxorg/ace-builds/d979a82c6198802cfb6df5be236379d29f116b2d/src-noconflict/theme-twilight.js > $@

ansi_up.js:
	curl https://raw.githubusercontent.com/drudru/ansi_up/aa1627cbc4aababee74a2f55e7525a14610bbd74/ansi_up.js > ansi_up.js

viz.js:
	curl -L https://github.com/mdaines/viz.js/releases/download/v2.1.2/viz.js > viz.js

viz.render.js:
	curl -L https://github.com/mdaines/viz.js/releases/download/v2.1.2/full.render.js > viz.render.js