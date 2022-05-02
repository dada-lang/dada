"use strict";

/**
 * Variable with regex to validate blank lines
 * @name removeBlankLines
 * @var
 * @returns {Regex}
 */

var blankLines = new RegExp(/(^[ \t]*\n)/, "gm");

/**
 * removeBlankLines
 * Remove blank lines from a string.
 *
 * @name removeBlankLines
 * @function
 * @param {String} input The input string.
 * @returns {String} The result string (without blank lines).
 */
var removeBlankLines = function removeBlankLines(input) {
  return input.replace(blankLines, "");
};

module.exports = removeBlankLines;