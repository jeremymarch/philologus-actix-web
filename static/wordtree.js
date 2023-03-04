/**
 * Copyright 2006-2011 Jeremy March.    All rights reserved.
 */

/* global */
const globalWordTrees = [];

/* global */
const keyDelay = 350; // to prevent each key press from triggering a query for fast typers
const cacheLimit = 500;
const debug = false;
const mouseWheelSpeedFactor = 10; // between 4 and 12 seem to work best
const keyScrollAccel = 4;

let browser;
if (window.navigator.userAgent.toLowerCase().indexOf('ie') !== -1) { browser = 'ie'; } else if (window.navigator.userAgent.toLowerCase().indexOf('webkit') != -1) { browser = 'safari'; } else { browser = 'firefox'; }

let platform;
if (window.navigator.userAgent.indexOf('iPhone') !== -1) { platform = 'iphone'; } else if (window.navigator.userAgent.indexOf('iPad') != -1) { platform = 'ipad'; } else if (window.navigator.platform.toLowerCase().indexOf('mac') != -1) { platform = 'mac'; } else if (window.navigator.platform.toLowerCase().indexOf('linux') != -1) { platform = 'linux'; } else { platform = 'windows'; }

function lookupWT (idPrefix) {
  for (let i = 0; i < globalWordTrees.length; i++) {
    if (globalWordTrees[i][0] === idPrefix) { return globalWordTrees[i][1]; }
  }

  return null;
}

function morphClick () {
  const matches = this.id.match(/(.*)MorphButton/);
  if (matches && matches[1]) {
    const w = lookupWT(matches[1]);
    if (w) {
      if (!w.morphMode) {
        w.asYouType = false;
        w.url = 'morphserv.php';
        w.morphMode = true;
        this.style.color = '#888';
        this.style.backgroundColor = 'white';
        this.style.borderColor = 'red';
      } else {
        w.asYouType = true;
        w.url = 'query';
        w.morphMode = false;
        this.style.color = '#EEE';
        this.style.backgroundColor = '#CCC';
        this.style.borderColor = '#AAA';
      }
      w.clearCache();
      // alert(w.morphMode);
    }
  }
}

function destroyWT () {
  const p = this.div.parentNode;
  p.removeChild(this.div);
  let w = lookupWT(this.idPrefix);

  w = null;

  for (let i = 0; i < globalWordTrees.length; i++) {
    if (globalWordTrees[i][0] === this.idPrefix) {
      globalWordTrees.splice(i, 1);
      break;
    }
  }
  return null;
}

function wordtree (idPrefix, width, height) {
  // alert("abc1");
  this.showMorph = false;
  this.mode = 'context';
  this.morphMode = false;
  this.asYouType = true;
  this.width = width;
  this.bgcolor = '#ffffff';
  this.selectedRow = null;
  this.accelTimeout = null;
  this.lastRequestTime = null;
  this.lastKeyTimeout = null;
  this.indentWidth = 15; // for tree branches
  this.conTopOffset = 79;

  /*
    At first we were using the system default rate for scrolling when
    the arrow keys were held down.    It might be more consistent across
    systems and smoother to use a timer to control how fast scrolling
    occurs.
    */
  this.scrollTimer = false; // whether to use the scroll timer or the old way
  this.scrollTimerKeyDown = false;
  this.scrollTimerTimeout = null;
  this.scrollTimerDelay = 1500; // time before the timer is invoked
  this.scrollTimerRate = 100; // rate of scroll
  this.scrollTimerStep = 1; // step of scroll

  this.step = 1;

  // whether to automatically focus the entry when wt "has focus", disable for iphones, ipads, etc.
  if (platform === 'ipad' || platform === 'iphone' || (navigator.maxTouchPoints && navigator.maxTouchPoints > 1)) { this.autofocus = false; } else { this.autofocus = true; }

  this.maxWords = 100;
  this.url = 'query';
  this.idPrefix = idPrefix;

  this.params = {};

  this.columnOffsets = [0, 0, 0];
  this.columns = 1;

  this.rowCount = 0;
  this.pageUp = 0; // for scrolling up
  this.page = 0; // for scrolling down
  this.nextPageRequestPending = false;
  this.prevPageRequestPending = false;
  this.lastPage = false;
  this.lastPageUp = false;
  this.blockScroll = false; // fixes a bug in webkit where next page was requested while loading 0 page

  const d = document.createElement('div');
  this.div = d;
  d.id = idPrefix;
  d.classList.add('wordtree');

  const ti = document.createElement('div');
  ti.style.position = 'absolute';
  ti.style.top = '17px';
  ti.style.left = '6px';
  ti.style.width = this.width - 33 + 'px';
  ti.innerHTML = 'Title';
  this.title = ti;

  this.div.appendChild(ti);

  const input = document.createElement('input');
  input.ariaLabel = 'Enter prefix for word';
  input.style.width = this.width - 67 + 'px';
  input.style.position = 'absolute';
  input.style.top = '66px';
  input.style.left = '12px';
  input.setAttribute('autocomplete', 'off');
  input.setAttribute('autocorrect', 'off');
  input.setAttribute('autocapitalize', 'off');
  input.setAttribute('spellcheck', 'false');
  // input.setAttribute("lang", "gr");
  input.id = idPrefix + 'Entry';
  this.entry = input;

  const ftcheck = document.createElement('input');
  ftcheck.type = 'checkbox';
  ftcheck.ariaLabel = 'full-text toggle';
  ftcheck.style.position = 'absolute';
  ftcheck.style.top = '42px';
  ftcheck.style.left = '231px';
  ftcheck.style.display = 'none';
  ftcheck.id = idPrefix + 'FTCheck';
  // ftcheck.onclick = ftclicked;
  this.ft = ftcheck;

  const ftlabel = document.createElement('label');
  ftlabel.setAttribute('for', idPrefix + 'FTCheck');
  ftlabel.style.position = 'absolute';
  ftlabel.style.top = '46px';
  ftlabel.style.left = '213px';
  ftlabel.innerHTML = 'FT';
  ftlabel.style.display = 'none';
  ftlabel.style.fontFamily = 'helvetica, arial, sans-serif';
  ftlabel.style.zIndex = 999;

  const loading = document.createElement('div');
  loading.id = this.idPrefix + 'Loading';
  // loading.src = "images/loading2.gif";
  loading.style.position = 'absolute';
  loading.style.top = '39px';
  loading.style.right = '44px';
  loading.style.display = 'none';
  loading.style.height = '18px';
  loading.style.width = '18px';
  loading.style.zIndex = 999;
  // var loading = document.createElement("div");
  // loading.innerHTML = '<svg class="spinner" viewBox="0 0 50 50"><circle class="path" cx="25" cy="25" r="20" fill="none" stroke-width="10"></circle></svg>';
  loading.innerHTML = '<div class="lds-spinner"><div></div><div></div><div></div><div></div><div></div><div></div><div></div><div></div><div></div><div></div><div></div><div></div></div>';

  this.loading = loading;

  if (this.showMorph) {
    this.morphClick = morphClick;

    const morph = document.createElement('div');
    morph.style.width = '40px';
    morph.style.height = '13px';
    morph.style.color = '#EEE';
    morph.style.textAlign = 'center';
    morph.style.backgroundColor = '#CCC';
    morph.style.border = '2px solid #AAA';
    morph.style.position = 'absolute';
    morph.style.top = '42px';
    morph.style.right = '10px';
    morph.style.fontSize = '10pt';
    morph.style.zIndex = 9;
    morph.style.cursor = 'pointer';
    // morph.style.verticalAlign = "-10px";
    morph.innerHTML = 'morph';
    morph.id = idPrefix + 'MorphButton';
    morph.onclick = this.morphClick;
    this.div.appendChild(morph);
  }

  this.closedNodeImg = 'images/plus.gif';
  this.openNodeImg = 'images/minus.gif';

  input.onkeydown = wordtree_ondown;
  input.onkeyup = wordtree_onup;

  this.div.appendChild(ftcheck);
  this.div.appendChild(ftlabel);
  this.div.appendChild(input);
  this.div.appendChild(loading);

  const con = document.createElement('div');
  con.classList.add('WordContainer');
  con.id = idPrefix + 'Container';

  con.style.top = this.conTopOffset + 'px';
  con.style.width = this.width - 20 + 'px';

  /*
    if (con.addEventListener)
    {
            con.addEventListener('DOMMouseScroll', onMouseWheel, false);
            con.addEventListener("mousewheel", onMouseWheel, false);
    }
    else if (con.attachEvent) {
            con.attachEvent("onmousewheel", onMouseWheel);
    }
    */

  con.onscroll = conOnScroll;
  if (platform === 'ipad' || platform === 'iphone' || (navigator.maxTouchPoints && navigator.maxTouchPoints > 1)) {
    con.ontouchstart = onTouch;
  }
  const wt = this;
  d.onclick = function () { if (wt.entry && wt.autofocus) wt.entry.focus(); };
  // d.oncontextmenu = function () { if (typeof wt.onContextMenu == "function") wt.onContextMenu(); return false; };
  this.div.appendChild(con);

  this.con = con;

  globalWordTrees.push([idPrefix, this]);

  this.show = wordtree_show;
  this.close = destroyWT;
  this.setHeight = setHeight;
  this.setWidth = setWidth;
  this.centerSelectedRow = centerSelectedRow;
  this.refresh = refreshWordTree;
  this.refreshWithRows = refreshWithRows;
  this.requestNextPage = requestNextPage;
  this.requestPrevPage = requestPrevPage;
  this.clearWordTree = clearWordTree;
  this.cellInit = cellInit;
  this.requestRows = requestRows;
  this.makeQuery = makeQueryString;
  this.clearCache = clearCache;
  this.cache = null;
  this.cacheCount = 0;
  this.getColumnValues = getColumnValues;

  this.f = null;
  this.dragDest = null;

  this.onEnterActivate = null;
  this.onClickActivate = null;
  this.onSelectionChanged = null;
  this.onAddWord = null;
  this.onDeleteWord = null;
  // this.onContextMenu = context;

  this.openAllNodes = openAllNodes;
  this.closeAllNodes = closeAllNodes;

  this.setHeight(height);
  this.setWidth(width);

  function makeQueryString (paramsObj) {
    let json = '{';

    for (prop in paramsObj) { json += '"' + prop + '":"' + paramsObj[prop] + '",'; }

    json = json.replace(/[,]+$/, ''); // trim trailing comma
    json += '}';

    return json;
  }

  function cellInit () {
    this.centerSelectedRow();
  }

  function clearWordTree () {
    const con = this.con;
    let a = con.firstChild;
    while (a) {
      const b = a.nextSibling;
      con.removeChild(a);
      a = b;
    }
    this.selectedRow = null;
    this.rowCount = 0;
  }

  function setHeight (height) {
    this.height = height;
    this.div.style.height = (height - 7) + 'px';
    this.con.style.height = (height - 22 - this.conTopOffset) + 'px';
  }

  function setWidth (width) {
    this.width = width;
    this.con.style.width = (width - 20) + 'px';
    if (this.entry) { this.entry.style.width = (width - 67) + 'px'; }
    this.div.style.width = width - 1 + 'px';
  }

  function refreshWithRows (rows) {
    this.lastRequestTime = 0; // defeat sequence check
    procResponse(rows, 'success');
  }

  function refreshWordTree () {
    const requestTime = new Date().getTime();
    this.loading.style.display = 'block';

    if (checkCache(this)) { return; }

    if (this.entry) { this.params.w = this.entry.value; }
    let query = makeQueryString(this.params);
    query = encodeURIComponent(query);

    if (fullText) {
      fullTextRequest(this.entry.value);
    } else {
      // encodeURI... only required for IE--Mozilla did it automatically
      const url = this.url + '?n=' + (this.maxWords + 1) + '&idprefix=' + this.idPrefix + '&x=' + Math.random() + '&requestTime=' + requestTime + '&page=' + 0 + '&mode=' + this.mode + '&query=' + query;
      // console.log(url);
      requestRows(url);
    }
  }

  function requestNextPage () {
    const requestTime = new Date().getTime();
    this.loading.style.display = 'block';

    if (this.entry) { this.params.w = this.entry.value; }
    let query = makeQueryString(this.params);
    query = encodeURIComponent(query);

    // encodeURI... only required for IE--Mozilla did it automatically
    const url = this.url + '?n=' + (this.maxWords + 1) + '&idprefix=' + this.idPrefix + '&x=' + Math.random() + '&requestTime=' + requestTime + '&page=' + (parseInt(this.page) + 1) + '&mode=' + this.mode + '&query=' + query;

    requestRows(url);
  }

  function requestPrevPage () {
    const requestTime = new Date().getTime();
    this.loading.style.display = 'block';

    if (this.entry) { this.params.w = this.entry.value; }
    let query = makeQueryString(this.params);
    query = encodeURIComponent(query);

    // encodeURI... only required for IE--Mozilla did it automatically
    const url = this.url + '?n=' + (this.maxWords + 1) + '&idprefix=' + this.idPrefix + '&x=' + Math.random() + '&requestTime=' + requestTime + '&page=' + (parseInt(this.pageUp) - 1) + '&mode=' + this.mode + '&query=' + query;

    requestRows(url);
  }

  function wordtree_show (parent) {
    parent.appendChild(this.div);
  }

  function onTouch (e) {
    const wt = lookupWT('test1');
    if (!wt) {
      return;
    }

    if (!wt.autofocus) {
      wt.entry.blur();
    }
  }

  function conOnScroll (e) {
    // increase this number to fetch next page earlier (for slower connections if there is a lag when you hit the end of the last page),
    // decrease if it's being fetched to soon
    const whenToGetNextPage = 400;

    // this = the container element
    const match = /(.*)Container/.exec(this.id);
    if (!match) {
      return;
    }

    const wt = lookupWT(match[1]);
    if (!wt) {
      return;
    }

    // don't request another page if we're on the last page OR if another nextPageRequest is pending.
    // don't increase page until it is received and appropriate
    if (wt.con.scrollTop > wt.con.scrollHeight - wt.con.offsetHeight - whenToGetNextPage && !wt.nextPageRequestPending && !wt.lastPage && !wt.blockScroll) {
      wt.nextPageRequestPending = true;
      wt.requestNextPage();
    } else if (wt.mode === 'context' && wt.con.scrollTop < whenToGetNextPage && !wt.prevPageRequestPending && !wt.lastPageUp && !wt.blockScroll) {
      wt.prevPageRequestPending = true;
      wt.requestPrevPage();
    }
  }

  function onMouseWheel (e) {
    // v. http://www.switchonthecode.com/tutorials/javascript-tutorial-the-scroll-wheel
    const el = this;
    const amount = e.detail ? e.detail * mouseWheelSpeedFactor : e.wheelDelta / 40 * mouseWheelSpeedFactor * -1;

    el.scrollTop += amount;

    return cancelEvent(e);
  }

  function cancelEvent (e) {
    e = e || window.event;
    if (e.stopPropagation) { e.stopPropagation(); }
    if (e.preventDefault) { e.preventDefault(); }
    e.cancelBubble = true;
    e.cancel = true;
    e.returnValue = false;
    return false;
  }

  function wordtree_onup (ev) {
    if (!ev) { ev = window.event; }

    const key = ev.keyCode;

    const match = /(.*)Entry/.exec(this.id);
    if (!match) { return; }
    const idPrefix = match[1];

    const wt = lookupWT(idPrefix);
    if (!wt) { return; }

    clearTimeout(wt.accelTimeout);
    wt.accelTimeout = null;
    wt.step = 1;
    wt.downkey = false;

    if (wt.scrollTimer) {
      if (wt.scrollTimerTimeout) { clearTimeout(wt.scrollTimerTimeout); }
      wt.scrollTimerTimeout = false;
      wt.scrollTimerKeyDown = false;
    }
    /*
                //temp for testing paging without actually paging
                if (key == 78)
                {
                        wt.prevPageRequestPending = true;
                        wt.requestPrevPage();
                }
        */

    if (key === 40 || key === 38) {
      keydown = false;
      if (typeof wt.onSelectionChanged === 'function' && wt.selectedRow) {
        wt.onSelectionChanged(wt.params.lexicon, getColumnValues(wt.selectedRow));
      }
    } else if (!ev.ctrlKey && (((key >= 48 && key <= 90) || key === 8 || key === 46) || key === 0) && wt.asYouType) {
      // block fast typers from making requests for every keystroke
      wt.page = 0;
      wt.selectedRow = null;

      if (wt.lastKeyTimeout) { clearTimeout(wt.lastKeyTimeout); }
      wt.lastKeyTimeout = setTimeout("lookupWT('" + wt.idPrefix + "').refresh();", keyDelay);
    } else if (key === 27) { // esc
      /*
            if (wt.entry.value == "")
    {
        alert("a: " + wt.entry.value);
                wt.params.tag_id = 0;
            }
            */
      /*
wt.entry.value = "";
wt.page = 0;
wt.selectedRow = null;

//block fast typers from making requests for every keystroke
if (wt.lastKeyTimeout)
    clearTimeout(wt.lastKeyTimeout);
wt.lastKeyTimeout = setTimeout("var a = lookupWT('" + wt.idPrefix + "'); a.refresh(); if (a.entry && a.autofocus) a.entry.focus()", keyDelay);
    */
    }
  }

  function wordtree_ondown (ev) {
    const key = ev.keyCode;

    if (key === 17) {
      return;
    }

    const match = /(.*)Entry/.exec(this.id);
    if (!match) {
      return;
    }
    const idPrefix = match[1];

    const wt = lookupWT(idPrefix);
    if (!wt) {
      return;
    }

    if (key === 39) { // right arrow: open row
      if (wt.selectedRow && wt.selectedRow.id) {
        // first open row
        if (!openNode(wt, wt.selectedRow.id)) {
          // on second key press open row's children
          wt.openAllNodes(wt.selectedRow.nextSibling);
        }
      }
    } else if (key === 37) { // left arrow: close row
      if (wt.selectedRow && wt.selectedRow.id) {
        closeNode(wt, wt.selectedRow.id);
        wt.closeAllNodes(wt.selectedRow.nextSibling);
        /*
            if (!closeNode(wt, wt.selectedRow.id))
            {
                wt.closeAllNodes(wt.selectedRow.nextSibling);
            }
            */
      }
    } else if (key === 38) {
      if (wt.scrollTimer) {
        if (!wt.scrollTimerKeyDown) {
          if (!wt.scrollTimerTimeout && wt.step > 1) { wt.scrollTimerTimeout = setTimeout(function () { wt.scrollTimerKeyDown = true; move(1, wt, wt.step); }, wt.scrollTimerDelay); }

          move(1, wt, wt.step);
        }
      } else {
        move(1, wt, wt.step);
        // accelerate
        if (!wt.downkey) {
          wt.accelTimeout = setTimeout("var a = lookupWT('" + wt.idPrefix + "'); a.step = 2; a.accelTimeout = setTimeout(\"lookupWT('test1').step = keyScrollAccel\", 2000)", 2000);
        }
      }
    } else if (key === 40) {
      if (wt.scrollTimer) {
        if (!wt.scrollTimerKeyDown) {
          if (!wt.scrollTimerTimeout && wt.step > 1) { wt.scrollTimerTimeout = setTimeout(function () { wt.scrollTimerKeyDown = true; move(-1, wt, wt.step); }, wt.scrollTimerDelay); }

          move(-1, wt, wt.step);
        }
      } else {
        move(-1, wt, wt.step);
        // accelerate
        if (!wt.downkey) {
          wt.accelTimeout = setTimeout("var a = lookupWT('" + wt.idPrefix + "'); a.step = 2; a.accelTimeout = setTimeout(\"lookupWT('test1').step = keyScrollAccel\", 2000)", 2000);
        }
      }
    }

    if (!wt.downkey) {
      // put non-repeating downkey stuff here
      if (ev.ctrlKey) {
        if (key === 65) { // a
          if (typeof wt.onAddWord === 'function' && wt.selectedRow) {
            wt.onAddWord(wt.params.lexicon, getColumnValues(wt.selectedRow));

            // block default hot keys, like bookmark, etc
            ev.returnValue = false;
            if (typeof ev.preventDefault === 'function') { ev.preventDefault(); }
          }
        }
        if (key === 68) { // d
          if (typeof wt.onDeleteWord === 'function' && wt.selectedRow) {
            wt.onDeleteWord(getColumnValues(wt.selectedRow)[1], getColumnValues(wt.selectedRow)[2], wt.params.tag_id);

            // block default hot keys, like bookmark, etc
            ev.returnValue = false;
            if (typeof ev.preventDefault === 'function') { ev.preventDefault(); }
          }
        }
        if (key === 69) { // e
          if (wt.selectedRow) { toggleNode(wt, wt.selectedRow.id); }

          // block default hot keys, like bookmark, etc
          ev.returnValue = false;
          if (typeof ev.preventDefault === 'function') { ev.preventDefault(); }
        }
      }

      switch (key) {
        case 17: // control
          ev.returnValue = false;
          if (typeof ev.preventDefault === 'function') { ev.preventDefault(); }
          return false;
        case 13: // enter
          // alert(wt.asYouType);
          if (!wt.asYouType) {
            // alert(wt.url);
            wt.refresh();
          } else if (typeof wt.onEnterActivate === 'function' && wt.selectedRow) {
            // alert("a" + wt.url);
            wt.onEnterActivate(wt.params.lexicon, getColumnValues(wt.selectedRow));
          }
          break;
        case 27: // esc
          // clear tags if no text to clear
          if (wt.entry.value === '') {
            wt.params.tag_id = 0;
          }
          wt.entry.value = '';
          wt.page = 0;
          wt.selectedRow = null;

          // block fast typers from making requests for every keystroke
          if (wt.lastKeyTimeout) { clearTimeout(wt.lastKeyTimeout); }
          wt.lastKeyTimeout = setTimeout("var a = lookupWT('" + wt.idPrefix + "'); a.refresh(); if (a.entry && a.autofocus) a.entry.focus()", keyDelay);
          break;
        default:
          break;
      }
    }

    wt.downkey = true;
    if (!ev.ctrlKey && !ev.metaKey && (wt.params.lexicon === 'lsj' || wt.params.lexicon === 'slater')) {
      return transliterateKey(ev);
    } else {
      return true;
    }
  }

  function move (upDown, wt, step) {
    const con = wt.con;
    let n;
    let lastGoodn = null;

    if (!wt.selectedRow) {
      n = con.firstChild;
    } else if (upDown > 0) { // up
      if (wt.selectedRow !== con.firstChild) {
        n = wt.selectedRow;
        /*
                do
                {
                        n = stepUp(n);
                } while (n && n.parentNode.style.display == "none");
                */
        for (var i = 0; n && i < step; i++) {
          lastGoodn = n;
          n = stepUp(n);
        }

        if (!n) { n = lastGoodn; }
      }
    } else { // down
      if (wt.selectedRow !== con.lastChild) {
        n = wt.selectedRow;
        /*
                do
                {
                        n = stepDown(n);
                } while (n && n.parentNode.style.display == "none");
                */
        let i = 0;
        for (; n && i < step; i++) {
          lastGoodn = n;
          n = stepDown(n);
        }

        if (!n) { n = lastGoodn; }
      }
    }

    if (!n) {
      return;
    }

    if (wt.selectedRow) {
      if (wt.selectedRow.classList.contains('selectedRowClass')) {
        wt.selectedRow.classList.remove('selectedRowClass');
      }
    }

    n.classList.add('selectedRowClass');

    if (wt.dragSource) {
      n.setAttribute('draggable', true);
      n.addEventListener('dragstart', wt.dragStartFunc, false);
    }

    let childOffset = n.offsetTop;
    let temp = n.parentNode;

    // var t = n.offsetTop;
    // var s = "n: " + n.offsetTop;

    // this is needed because n.offSetTop does not work if the node is buried in 2 deep -Children divs
    while (temp.id.indexOf('Container') === -1) { // this means we can't use "Container" as a prefix name
      // s += " + " + temp.id + " (" + temp.offsetTop + ") ";
      // t += " + " + temp.offsetTop;

      childOffset += temp.offsetTop;
      temp = temp.parentNode; // or temp.offsetParent? both seem to work
    }
    // document.getElementById("t").innerHTML = s + "<br><br>" + t + " = " + (childOffset);

    // scroll as you go up
    if (childOffset < con.scrollTop + 30) { // was 15
      con.scrollTop = childOffset - 30; // was 18
    }

    // scroll as you go down
    if (childOffset > con.scrollTop + con.offsetHeight - 50) { // was 30
      con.scrollTop = childOffset - con.offsetHeight + 50; // was 40
    }

    wt.selectedRow = n;

    function stepDown (n) {
      if (!n) { return null; }

      // now with hierarchies the upper node could the start of a Child container or we could be leaving a child container
      let p = n.parentNode;

      while (n) {
        n = n.nextSibling;

        if (n && n.id.indexOf('Children') !== -1) {
          if (n.style.display === 'none') { continue; }

          n = n.firstChild;
        }
        break;
      }

      if (!n) {
        while (p && p.id.indexOf('Children') !== -1) {
          if (p.nextSibling) {
            n = p.nextSibling;
            break;
          }
          p = p.parentNode;
        }
      }
      return n;
    }

    function stepUp (n1) {
      if (!n1) { return null; }

      var n = n1.previousSibling;
      while (n && n.id.indexOf('Children') !== -1) {
        if (n.style.display === 'none') {
          var n = n.previousSibling;
          continue;
        }

        n = n.lastChild;
      }

      if (!n && n1.parentNode.id.indexOf('Container') === -1) {
        n = n1.parentNode.previousSibling;
      }
      return n;
    }

    if (wt.scrollTimer && wt.scrollTimerKeyDown) {
      setTimeout(function () { move(upDown, wt, wt.scrollTimerStep); }, wt.scrollTimerRate);
    }
  }

  /*
    row object key:
    i = id
    r = row an array for the different columns
    h = has_children
    c = array of row objects for this row's children
    o = open row
    s = sequence

    eventually lexicon, query, and tag_id will be put into a single field for requests
    */
  // a two column response
  // var resp2 = '{"wtprefix":"test1","container":"test1Container","requestTime":"99999","selectId":"0","page":"0","lastPage":"0","lastPageUp":"1","scroll":"top","query":"a","cols":"2","arrOptions":[{"i":1,"r":["Α","abc"]},{"i":5,"r":["ἃ","abc"]},{"i":2,"r":["ἀ1","abc"]},{"i":20395,"r":["α1","abc"]},{"i":3,"r":["ἀ2","abc"]},{"i":102761,"r":["α2","abc"]},{"i":4,"r":["ἆ3","abc"]},{"i":6,"r":["ἄα","abc"]},{"i":8,"r":["ἀάβακτοι","abc"]},{"i":9,"r":["ἀαγής","abc"]}]}';
  // var resp2 = '{"error":"","wtprefix":"test1","nocache":"0","container":"test1Container","requestTime":"1631672832851","selectId":"0","page":"0","lastPage":"0","lastPageUp":"1","scroll":"top","query":"","arrOptions":[{"i":0,"r":["Α α",0,0]},{"i":1,"r":["ἀ-",1,0]},{"i":2,"r":["ἀ-",2,0]},{"i":3,"r":["ἆ",3,0]},{"i":4,"r":["ἃ ἃ",4,0]},{"i":5,"r":["ἄα",5,0]},{"i":6,"r":["ἀάατος",6,0]},{"i":7,"r":["ἀάβακτοι",7,0]},{"i":8,"r":["ἀαγής",8,0]},{"i":9,"r":["ἄαδα",9,0]},{"i":10,"r":["ἀάζω",10,0]},{"i":11,"r":["ἄαθι",11,0]},{"i":12,"r":["ἀάκατος",12,0]},{"i":13,"r":["ἀακίδωτος",13,0]},{"i":14,"r":["ἀάλιον",14,0]},{"i":15,"r":["ἀανές",15,0]},{"i":16,"r":["ἄανθα",16,0]},{"i":17,"r":["ἀάπλετος",17,0]},{"i":18,"r":["ἄαπτος",18,0]},{"i":19,"r":["ἄας",19,0]},{"i":20,"r":["ἀασιφόρος",20,0]},{"i":21,"r":["ἀασιφρονία",21,0]},{"i":22,"r":["ἀασιφροσύνη",22,0]},{"i":23,"r":["ἀάσκει",23,0]},{"i":24,"r":["ἀασμός",24,0]},{"i":25,"r":["ἀάσπετος",25,0]},{"i":26,"r":["ἀάστονα",26,0]},{"i":27,"r":["ἀατήρ",27,0]},{"i":28,"r":["ἄατος",28,0]},{"i":29,"r":["ἄατος",29,0]},{"i":30,"r":["ἀάτυλον",30,0]},{"i":31,"r":["ἀάω",31,0]},{"i":32,"r":["ἄβα",32,0]},{"i":33,"r":["ἄβαγνα",33,0]},{"i":34,"r":["ἀβαθής",34,0]},{"i":35,"r":["ἄβαθρος",35,0]},{"i":36,"r":["ἀβαίνω",36,0]},{"i":37,"r":["ἀβακέω",37,0]},{"i":38,"r":["ἀβακηνούς",38,0]},{"i":39,"r":["ἀβακής",39,0]},{"i":40,"r":["ἀβάκητον",40,0]},{"i":41,"r":["ἀβακίζομαι",41,0]},{"i":42,"r":["ἀβάκιον",42,0]},{"i":43,"r":["ἀβακίσκος",43,0]},{"i":44,"r":["ἀβακλή",44,0]},{"i":45,"r":["ἀβακοειδής",45,0]},{"i":46,"r":["ἄβακτον",46,0]},{"i":47,"r":["ἀβάκχευτος",47,0]},{"i":48,"r":["ἀβακχίωτος",48,0]},{"i":49,"r":["ἄβαλε",49,0]},{"i":50,"r":["ἀβαμβάκευτος",50,0]},{"i":51,"r":["ἄβαξ",51,0]},{"i":52,"r":["ἀβάπτιστος",52,0]},{"i":53,"r":["ἄβαπτος",53,0]},{"i":54,"r":["ἀβαρβάριστος",54,0]},{"i":55,"r":["ἀβαρής",55,0]},{"i":56,"r":["ἄβαρις",56,0]},{"i":57,"r":["ἀβασάνιστος",57,0]},{"i":58,"r":["ἀβασίλευτος",58,0]},{"i":59,"r":["ἀβασκάνιστος",59,0]},{"i":60,"r":["ἀβάσκανος",60,0]},{"i":61,"r":["ἀβάσκαντος",61,0]},{"i":62,"r":["ἀβάστακτος",62,0]},{"i":63,"r":["ἄβαστον",63,0]},{"i":64,"r":["ἀβατόομαι",64,0]},{"i":65,"r":["ἄβατος",65,0]},{"i":66,"r":["ἀβαφής",66,0]},{"i":67,"r":["ἄβδελον",67,0]},{"i":68,"r":["ἀβδέλυκτος",68,0]},{"i":69,"r":["Ἀβδηρίτης",69,0]},{"i":70,"r":["ἄβδης",70,0]},{"i":71,"r":["ἀβέβαιος",71,0]},{"i":72,"r":["ἀβεβαιότης",72,0]},{"i":73,"r":["ἀβέβηλος",73,0]},{"i":74,"r":["ἄβεις",74,0]},{"i":75,"r":["ἄβελλον",75,0]},{"i":76,"r":["ἀβελτέρειος",76,0]},{"i":77,"r":["ἀβελτερεύομαι",77,0]},{"i":78,"r":["ἀβελτερία",78,0]},{"i":79,"r":["ἀβελτεροκόκκυξ",79,0]},{"i":80,"r":["ἀβέλτερος",80,0]},{"i":81,"r":["ἀβέρβηλον",81,0]},{"i":82,"r":["ἀβηδών",82,0]},{"i":83,"r":["ἀβήρελ",83,0]},{"i":84,"r":["ἀβηροῦσιν",84,0]},{"i":85,"r":["ἀβίαστος",85,0]},{"i":86,"r":["ἀβίβαστος",86,0]},{"i":87,"r":["ἀβίβλης",87,0]},{"i":88,"r":["ἄβιδα",88,0]},{"i":89,"r":["ἄβιν",89,0]},{"i":90,"r":["ἄβιος",90,0]},{"i":91,"r":["ἄβιος",91,0]},{"i":92,"r":["ἀβίοτος",92,0]},{"i":93,"r":["ἀβίυκτον",93,0]},{"i":94,"r":["ἀβιωτοποιός",94,0]},{"i":95,"r":["ἀβίωτος",95,0]},{"i":96,"r":["ἀβλάβεια",96,0]},{"i":97,"r":["ἀβλαβής",97,0]},{"i":98,"r":["ἀβλαβία",98,0]},{"i":99,"r":["ἀβλαβύνιον",99,0]},{"i":100,"r":["ἄβλαπτος",100,0]}]}';

  function procResponse (str, status) {
    // var start = Date.now();

    // if (status != "success")
    // return;

    let returnObj;
    // str = resp2;
    // console.log("Proc Response: " + str);
    // if (str.indexOf("test4") > -1)
    // alert(str);
    try {
      if (typeof JSON !== 'undefined') {
        returnObj = JSON.parse(str);
      } else {
        // console.log("browser does not support json decode");
        return;// returnObj = eval("(" + str + ")");
      }
    } catch (e) { if (debug) alert(e.message + '\n' + str); return; };

    if (!returnObj) {
      return;
    }

    const wt = lookupWT(returnObj.wtprefix);
    if (!wt) {
      return;
    }

    if (returnObj.mesgCode) {
      // alert(returnObj.mesg);
      const z = document.getElementById('mesg');
      if (z) {
        if (returnObj.mesgCode === 1) {
          z.style.backgroundColor = 'green';
          z.style.paddingTop = '12px';
          z.style.color = 'white';
          z.innerHTML = returnObj.mesg;
        } else {
          z.style.backgroundColor = 'red';
          z.style.paddingTop = '12px';
          z.style.color = 'white';
          z.innerHTML = returnObj.mesg;
        }
        z.style.display = 'block';
        setTimeout("document.getElementById('mesg').style.display = 'none'", 2000);
      }
      return;
    }

    wt.loading.style.display = 'none';
    wt.blockScroll = true;

    // if caching is activated, add result to cache
    if (wt.cache && returnObj.nocache === 0) { // the only one we don't cache is if if it's looked up via wordid
      wtAddResultToCache(wt, returnObj.query, str);
    }

    if (debug && returnObj.error) {
      wt.con.innerHTML = returnObj.error;
      wt.blockScroll = false;
      return;
    }

    // save original height here, for use if we're paging up.    See below.
    const saveHeight = wt.con.scrollHeight;

    // block result sets which come out of sequence
    if (wt.lastRequestTime > parseInt(returnObj.requestTime)) {
      // if (debug) {
      //         console.log("out of seq!");
      // }
      wt.blockScroll = false;
      return;
    } else {
      // set lastRequestTime to that of the last result set received
      wt.lastRequestTime = parseInt(returnObj.requestTime);
    }

    const returnedPage = parseInt(returnObj.page);

    // block pages which are repeats or out of order
    if (returnedPage < 0 && returnedPage >= wt.pageUp) {
      // if (debug) {
      //         console.log("wt.pageUp: " + wt.pageUp + "; returnObj.page: " + returnObj.page);
      // }

      wt.blockScroll = false;
      return;
    } else if (returnedPage > 0 && returnedPage <= wt.pageUp) {
      // if (debug) {
      //         console.log("wt.pageDown: " + wt.page + "; returnObj.pageDown: " + returnObj.page);
      // }

      wt.blockScroll = false;
      return;
    }

    const con = document.getElementById(returnObj.container);

    const arrOptions = returnObj.arrOptions;
    const len = arrOptions.length;

    if (returnedPage < 0) {
      wt.pageUp = returnedPage;
      wt.prevPageRequestPending = false;
    } else if (returnedPage === 0) {
      // reset these if we just refreshed a page 0
      wt.page = 0;
      wt.pageUp = 0;
    } else {
      wt.page = returnedPage;
      wt.nextPageRequestPending = false;
    }

    if (returnObj.parentid) { // && returnObj.treeOptions.length > 0)
      let i = 0;
      for (; i < len; i++) {
        // insertBefore, roots, and selectedId are both false because this function is only for inserting
        // non-top-level items lazily
        printTree(wt, con, arrOptions[i], 1, false, false, -1);
      }

      if (returnObj.selectId) {
        var node = document.getElementById(returnObj.selectId);
        if (node) {
          node.classList.add('selectedRowClass');
          wt.selectedRow = node;
        }
      }
      if (wt.entry && wt.autofocus) {
        wt.entry.focus();
      }

      wt.blockScroll = false;
    } else {
      // only set these if not a tree branch
      if (returnObj.lastPage === 1) { wt.lastPage = true; }
      if (returnObj.lastPage === 0) { wt.lastPage = false; }

      if (returnObj.lastPageUp === 1) { wt.lastPageUp = true; }
      if (returnObj.lastPageUp === 0) { wt.lastPageUp = false; }

      // delete old rows if this is page 0 and this isn't a tree
      if (returnedPage === 0 && !returnObj.parentid) {
        wt.clearWordTree();
      }

      if (debug) {
        const node = document.createElement('div');
        const text = document.createTextNode('Start Page ' + returnObj.page);
        node.appendChild(text);
        node.style.border = '1px solid white';
        node.id = '10blahWord';
        node.style.whiteSpace = 'nowrap';
        node.style.color = 'red';
        node.style.fontWeight = 'bold';
        if (returnObj.page < 0 && before) { con.insertBefore(node, before); } else { con.appendChild(node); }
      }
      let i = 0;
      for (; i < len; i++) {
        let insertBefore = false;
        if (returnedPage < 0) {
          insertBefore = true;
        }

        printTree(wt, con, arrOptions[i], 0, insertBefore, returnObj.roots, returnObj.selectId);
        // printRow(wt, con, arrOptions[i], 0, insertBefore, returnObj.roots, returnObj.selectId);
      }

      if (wt && returnObj.scroll === 'top') {
        wt.con.scrollTop = '0'; // chrome does not want me to use "px" here, just 0.
      } else if (wt && returnObj.scroll === 'bottom') {
        wt.con.scrollTop = wt.con.scrollHeight;
      } else if (wt && returnObj.selectId && returnedPage === 0) {
        // select middle word and scroll there
        const s = document.getElementById(wt.selectedRowId);
        if (s) {
          if (wt.selectedRow) {
            if (wt.selectedRow.classList.contains('selectedRowClass')) {
              wt.selectedRow.classList.remove('selectedRowClass');
            }
          }
          s.classList.add('selectedRowClass');
          wt.selectedRow = s;

          if (wt.dragSource) {
            s.setAttribute('draggable', true);
            s.addEventListener('dragstart', wt.dragStartFunc, false);
          }

          wt.centerSelectedRow();
        }
      }

      if (debug) {
        const node = document.createElement('div');
        const text = document.createTextNode('End Page ' + returnObj.page);
        node.appendChild(text);
        node.id = '10blahWord';
        node.style.border = '1px solid white';
        // node.setAttribute('class','treerow');
        node.style.whiteSpace = 'nowrap';
        node.style.color = 'red';
        node.style.fontWeight = 'bold';
        if (returnedPage < 0 && before) { con.insertBefore(node, before); } else { con.appendChild(node); }
      }

      if (returnedPage < 0) {
        // to keep scrollTop in same place as before when paging up.
        wt.con.scrollTop += (wt.con.scrollHeight - saveHeight);
      }
      wt.blockScroll = false; // fixes bug in webkit where next page was requested in middle of this function

      // request def
      if (wt.selectedRow && returnedPage === 0 && typeof wt.onSelectionChanged === 'function') { wt.onSelectionChanged(wt.params.lexicon, getColumnValues(wt.selectedRow)); }

      // var end = Date.now();
      // console.log("Time: %d", end - start);
    }

    // topLevelTreeRow tells us to indent rows with no children the width of the plus sign,
    // so they line up with the rows that do have children.
    function printRow (wt, con, rowItem, level, insertAtTop, topLevelTreeRow, selectedId) {
      const node = document.createElement('div');
      node.classList.add('nodestyle');
      node.setAttribute('rowid', wt.rowCount + wt.idPrefix);
      node.id = wt.rowCount++ + wt.idPrefix;

      node.onclick = onSelect;

      if (wt.dragDest) {
        node.addEventListener('dragover', wt.overFunc, false);
        node.addEventListener('dragenter', wt.enterFunc, false); // to get IE to work
        node.addEventListener('dragleave', wt.leaveFunc, false);
        node.addEventListener('drop', wt.dropFunc, false);
      }

      // for each column
      const rowLen = rowItem.length;
      for (let c = 0; c < rowLen; c++) {
        const d2 = document.createElement('div');
        d2.classList.add('nodestylecol');

        if (c !== 0 && wt.columnOffsets[c] === 0) { d2.style.display = 'none'; } else { d2.style.left = wt.columnOffsets[c] + 'px'; }

        if (level === 0) { d2.style.paddingLeft = 1 + (!rowItem.h && topLevelTreeRow ? 15 : 3) + 'px'; } else { d2.style.paddingLeft = (level * wt.indentWidth) + (rowItem.h ? 3 : 15) + 'px'; }

        if (c === 0 && rowItem.h) { // only for column 1
          const img = document.createElement('img');
          if (!rowItem.o) { img.src = wt.closedNodeImg; } else { img.src = wt.openNodeImg; }
          img.style.display = 'inline';
          img.style.paddingRight = '4px';
          img.style.cursor = 'pointer';
          img.id = node.id + 'Img';
          img.onclick = openCloseCon;
          d2.appendChild(img);
        }

        const text = document.createTextNode(rowItem[c]);
        d2.appendChild(text);
        node.appendChild(d2);
      }

      if (rowItem[1] === selectedId) {
        wt.selectedRowId = node.id;
      }

      const before = con.firstChild;
      if (insertAtTop && before) { con.insertBefore(node, before); } else { con.appendChild(node); }
    }

    function openCloseCon (ev) {
      ev.cancelBubble = true;
      if (ev.stopPropagation) ev.stopPropagation();

      // "this" is a reference to the img node
      const match = parseNodeImgId(this.id);
      if (!match) { return; }

      const wtprefix = match.wtPrefix;
      const wordid = match.id;
      const wt = lookupWT(wtprefix);

      if (wt) {
        toggleNode(wt, wordid + wtprefix);
      }
    }

    function onSelect (ev) {
      const res = parseNodeId(this.id);
      if (!res) { return; }

      const idPrefix = res.wtPrefix;

      const w = lookupWT(idPrefix);

      if (w.selectedRow) {
        w.selectedRow.setAttribute('draggable', false);

        if (w.selectedRow.classList.contains('selectedRowClass')) {
          w.selectedRow.classList.remove('selectedRowClass');
        }
      }

      this.classList.add('selectedRowClass');

      if (w.dragSource) {
        this.setAttribute('draggable', true);
        this.addEventListener('dragstart', w.dragStartFunc, false);
      }

      w.selectedRow = this;

      if (typeof w.onClickActivate === 'function') {
        w.onClickActivate(w.params.lexicon, getColumnValues(this));
        // console.log("select: " + w.params.lexicon + ", " + getColumnValues(this));
      }
      if (w.entry && wt.autofocus) { w.entry.focus(); }
    }

    function printTree (wt, con, treeRow, level, insertBefore, roots, selectedId) {
      printRow(wt, con, treeRow, level, insertBefore, roots, selectedId);

      if (treeRow.h && treeRow.c) {
        let node2 = document.createElement('div');
        node2.style.position = 'relative';
        if (!treeRow.o) { node2.style.display = 'none'; }
        node2 = con.appendChild(node2);
        node2.id = node2.previousSibling.id + 'Children';

        const treeRowLen = treeRow.c.length;
        for (let i = 0; i < treeRowLen; i++) {
          printTree(wt, node2, treeRow.c[i], level + 1, insertBefore, roots, selectedId);
        }
      }
    }
  }

  function toggleNode (wt, nodeid) {
    const imgId = nodeid + 'Img';
    const imgNode = document.getElementById(imgId);

    if (!imgNode) { // its ok for n to be null (lazy load)
      return;
    }

    if (imgNode.src.indexOf(wt.closedNodeImg) !== -1) {
      openNode(wt, nodeid);
    } else {
      closeNode(wt, nodeid);
    }
    if (wt.entry && wt.autofocus) { wt.entry.focus(); }
  }

  /**
     * Return true if node was opened.    False otherwise.
     * This lets us open all, on second keypress.
     */
  function openNode (wt, nodeid) {
    const childrenConId = nodeid + 'Children';
    const imgId = nodeid + 'Img';
    const n = document.getElementById(childrenConId);
    const imgNode = document.getElementById(imgId);

    if (!imgNode) { // its ok for n to be null, but not imgNode
      return true;
    }

    if (imgNode.src.indexOf(wt.openNodeImg) !== -1) { return false; }

    imgNode.src = wt.openNodeImg;

    if (n) {
      n.style.display = 'block';
    } else { // lazy load children...
      const requestTime = new Date().getTime();
      const childrenCon = document.createElement('div');
      childrenCon.style.position = 'relative';

      childrenCon.id = childrenConId;

      if (imgNode.parentNode.parentNode.nextSibling) { wt.con.insertBefore(childrenCon, imgNode.parentNode.parentNode.nextSibling); } else { wt.con.appendChild(childrenCon); }

      wt.params.root_id = getColumnValues(document.getElementById(nodeid))[1];

      if (wt.selectedRow) { wt.params.selectedid = wt.selectedRow.id; } // maybe this should just be nodeid?
      let query = wt.makeQuery(wt.params);
      query = encodeURIComponent(query);

      const url = wt.url + '?n=' + (wt.maxWords + 1) + '&idprefix=' + wt.idPrefix + '&x=' + Math.random() + '&requestTime=' + requestTime + '&page=' + wt.page + '&con=' + childrenConId + '&mode=' + wt.mode + '&query=' + query;

      requestRows(url);
      wt.params.root_id = '';
      wt.params.selectedid = '';
    }
    return true;
  }

  /**
     * Return true if node was closed.    False otherwise.
     * This lets us close all, on second keypress.
     */
  function closeNode (wt, nodeid) {
    const childrenConId = nodeid + 'Children';
    const imgId = nodeid + 'Img';
    const n = document.getElementById(childrenConId);
    const imgNode = document.getElementById(imgId);

    if (!n || !imgNode) {
      return true;
    }

    if (imgNode.src.indexOf(wt.closedNodeImg) !== -1) {
      return false;
    }

    n.style.display = 'none';
    imgNode.src = wt.closedNodeImg;

    return true;
  }

  function openAllNodes (node) {
    const children = node.childNodes;
    const regex = new RegExp('^[0-9]+' + this.idPrefix + '$');
    const childLen = children.length;
    for (let i = 0; i < childLen; i++) {
      if (children[i].childNodes.length > 0) { this.openAllNodes(children[i]); }

      if (regex.exec(children[i].id)) { openNode(this, children[i].id); }
    }
  }

  function closeAllNodes (node) {
    if (node) {
      const children = node.childNodes;
      const regex = new RegExp('^[0-9]+' + this.idPrefix + '$');
      const childLen = children.length;
      for (let i = 0; i < childLen; i++) {
        if (children[i].childNodes.length > 0) { this.closeAllNodes(children[i]); }

        if (regex.exec(children[i].id)) { closeNode(this, children[i].id); }
      }
    }
  }

  function requestRows (url) {
    microAjax({
      url,
      method: 'GET',
      success: procResponse,
      warning: procResponseError,
      error: null
    });
  }

  function procResponseError (str) {
    if (typeof JSON !== 'undefined') {
      const returnObj = JSON.parse(str);
      if (typeof (returnObj.error) !== 'undefined') {
        console.log('response error: ' + returnObj.error);
        // wt.loading.style.display = "none";
      }
    }
  }

  function checkCache (wt) {
    if (!wt.entry) { return false; }

    let queryKey = '';
    if (wt.entry.value === '') { queryKey = wt.params.lexicon + '' + wt.params.tag_id; } else { queryKey = wt.params.lexicon + wt.entry.value + wt.params.tag_id; }

    if (wt.cache && wt.cache[queryKey]) {
      wt.lastRequestTime = 0; // defeat sequence check
      procResponse(wt.cache[queryKey].str, 'success');
      // alert("here: true");
      return true;
    } else {
      // alert("false");
      return false; // not cached, request it
    }
  }

  function clearCache () {
    this.cache = [];
  }

  function wtAddResultToCache (wt, queryKey, str) {
    // the queryKey is the lexicon + the query word

    queryKey = wt.params.lexicon + queryKey + wt.params.tag_id;
    // alert("add: " + queryKey + "\n\n" + str);
    // if this query isn't in the cache
    if (!wt.cache[queryKey]) {
      // alert("here");
      // if we're at the cacheLimit remove the oldest item
      // (shorter keys are more useful results to have so maybe we should give them added importance?)
      // use cacheLength because assoc arrays have no length property and we don't want to have to count them each time
      if (cacheLimit && wt.cacheLength >= cacheLimit) {
        let prev = null;
        for (const x in wt.cache) {
          if (!wt.cache.hasOwnProperty(x)) { continue; }

          if (prev == null || wt.cache[x].time < wt.cache[prev].time) { prev = x; }
        }
        if (prev) {
          wt.cacheLength--;
          delete wt.cache[prev];
        }
      }
      wt.cacheLength++;
      wt.cache[queryKey] = [];
      wt.cache[queryKey].str = str;
      if (queryKey === wt.params.lexicon + 'a' || queryKey === wt.params.lexicon + 'α') { // put empty queries into the future, so they are never removed from cache
        const d = new Date();
        d.setDate(d.getDate() + 5);
        wt.cache[queryKey].time = d.getTime();
      } else { wt.cache[queryKey].time = new Date().getTime(); }
    } else if (queryKey !== wt.params.lexicon + 'a' || queryKey === wt.params.lexicon + 'α') { // don't reset timestamp for empty query
      // if it is in the cache, update the timestamp
      wt.cache[queryKey].time = new Date().getTime();
    }
  }

  function centerSelectedRow () {
    if (this.selectedRow) {
      // scroll to middle
      this.con.scrollTop = this.selectedRow.offsetTop - (this.con.offsetHeight / 2) + 30;
    }
  }
}

/*
wtprefix
rowscon = wtprefix + "Container"
imageid = word/root_id + wtprefix + "Img"
rowchidren = word/rootid + wtprefix + "Children"
*/
function parseNodeId (id) {
  let r = null;
  const match = /([0-9]+)(.*)/.exec(id);
  if (match) {
    r = {};
    r.id = match[1];
    r.wtPrefix = match[2];
  }
  return r;
}

function parseNodeImgId (id) {
  let r = null;
  const match = /([0-9]+)(.+)Img/.exec(id);
  if (match) {
    r = {};
    r.id = match[1];
    r.wtPrefix = match[2];
  }
  return r;
}

function getColumnValues (row) {
  const values = [];
  let col = row.firstChild;

  for (let i = 0; col; i++) {
    const match = /.*Img/.exec(col.firstChild.id); // in case it has a + or - image
    if (match) { values[i] = col.firstChild.nextSibling.nodeValue; } else { values[i] = col.firstChild.nodeValue; }
    col = col.nextSibling;
  }
  return values;
}
