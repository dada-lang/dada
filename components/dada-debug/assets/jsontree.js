var JSONTree = (function() { // eslint-disable-line no-unused-vars
  var escapeMap = {
    '&': '&amp;',
    '<': '&lt;',
    '>': '&gt;',
    '"': '&quot;',
    '\'': '&#x27;',
    '/': '&#x2F;',
  };

  var internalId = 0;
  var instances = 0;

  this.create = function(data, settings) {
    instances += 1;
    return '<div class="jstTree">' + _jsVal(data) + '</div>';
  };

  this.click = function(elem) {
    if (elem.nextElementSibling.className !== 'jstHiddenBlock') {
      var block = findNextWithClass(elem, 'jstBracket');
      var siblings = _nextUntil(block, block.id + '_end');
      _hide(elem, siblings);
      elem.className = 'jstExpand';
    } else {
      var block = findNextWithClass(elem, 'jstBracket');
      var hiddenElements = findNextWithClass(elem, 'jstHiddenBlock');
      var children = hiddenElements.children;
      for (var i = children.length; i > 0; i--) {
        var child = children[i - 1];
        block.after(child);
      }
      hiddenElements.remove();
      elem.className = 'jstCollapse';
    }
  };

  var findNextWithClass = function(element, clazz) {
    var next = element.nextElementSibling;
    while (true) {
      if (next.className === clazz) {
        return next;
      }
      next = next.nextElementSibling;
    }
  };

  var _id = function() {
    return instances + '_' + internalId++;
  };

  var _escape = function(text) {
    return text.replace(/[&<>'"]/g, function(c) {
      return escapeMap[c];
    });
  };

  var _jsVal = function(value) {
    var type = typeof value;
    switch (type) {
      case 'boolean':
        return _jsBool(value);
      case 'number':
        return _jsNum(value);
      case 'string':
        return _jsStr(value);
      default:
        if (value === null) {
          return _jsNull();
        } else if (value instanceof Array) {
          return _jsArr(value);
        } else {
          return _jsObj(value);
        }
    }
  };

  var _jsObj = function(object) {
    var id = _id();
    var elements = [];
    var keys = Object.keys(object);
    keys.forEach(function(key, index) {
      var html = [];
      html.push('<li class="jstItem">');
      if (_canCollapse(object[key])) {
        html.push(_collapseElem());
      }
      html.push(_property(key, object[key]));
      if (index !== keys.length - 1) {
        html.push(_comma());
      }
      html.push('</li>');
      elements.push(html.join(''));
    });
    var body = elements.join('');
    return _collection(_open('{', id), body, _close('}', id));
  };

  var _collapseElem = function() {
    var onClick = 'onclick="JSONTree.click(this); return false;"';
    return '<span class="jstCollapse" ' + onClick + '></span>';
  };

  var _canCollapse = function(data) {
    var type = typeof data;
    switch (type) {
      case 'boolean':
        return false;
      case 'number':
        return false;
      case 'string':
        return false;
      default:
        if (data === null) {
          return false;
        } else if (data instanceof Array) {
          return data.length > 0;
        } else {
          return Object.keys(data).length > 0;
        }
    }
  };

  var _collection = function(opening, data, closing) {
    if (data.length > 0) {
      return [
        opening,
        '<ul class="jstList">',
        data,
        '</ul>',
        closing,
      ].join('');
    } else {
      return opening + closing;
    }
  };

  var _jsArr = function(array) {
    var id = _id();
    var elements = [];
    array.forEach(function(element, index) {
      var html = ['<li class="jstItem">'];
      if (_canCollapse(element)) {
        html.push(_collapseElem());
      }
      html.push(_jsVal(element));
      if (index !== array.length - 1) {
        html.push(_comma());
      }
      html.push('</li>');
      elements.push(html.join(''));
    });
    var body = elements.join('');
    return _collection(_open('[', id), body, _close(']', id));
  };

  var _jsStr = function(value) {
    var jsonString = _escape(JSON.stringify(value));
    return _element(jsonString, {class: 'jstStr'});
  };

  var _jsNum = function(value) {
    return _element(value, {class: 'jstNum'});
  };

  var _jsBool = function(value) {
    return _element(value, {class: 'jstBool'});
  };

  var _jsNull = function() {
    return _element('null', {class: 'jstNull'});
  };

  var _property = function(name, value) {
    var escapedValue = _escape(JSON.stringify(name));
    var property = _element(escapedValue, {class: 'jstProperty'});
    var propertyValue = _jsVal(value);
    return [property + _colon(), propertyValue].join('');
  };

  var _colon = function() {
    return _element(': ', {class: 'jstColon'});
  };

  var _comma = function() {
    return _element(',', {class: 'jstComma'});
  };

  var _element = function(content, attrs) {
    var attrsStr = Object.keys(attrs).map(function(attr) {
      return ' ' + attr + '="' + attrs[attr] + '"';
    }).join('');
    return '<span' + attrsStr + '>' + content + '</span>';
  };

  var _open = function(sym, id) {
    return _element(sym, {id: 'opening_' + id, class: 'jstBracket'});
  };

  var _close = function(sym, id) {
    return _element(sym, {id: 'opening_' + id + '_end', class: 'jstBracket'});
  };

  var _nextUntil = function(elem, id) {
    var siblings = [];
    elem = elem.nextElementSibling;
    while (elem) {
      if (elem.id == id) {
        break;
      }
      siblings.push(elem);
      elem = elem.nextElementSibling;
    }
    return siblings;
  };

  var _hide = function(elem, siblings) {
    var wrapper = document.createElement('div');
    wrapper.className = 'jstHiddenBlock';
    siblings.forEach(function(s) {
      wrapper.appendChild(s);
    });
    elem.after(wrapper);
  };

  return this;
})();
