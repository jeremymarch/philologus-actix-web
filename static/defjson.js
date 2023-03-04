/**
 * Copyright 2006-2018 by Jeremy March.    All rights reserved.
 */
const defCache = [];
let defCacheLength = 0;
const defCacheLimit = 500;
const useDefCache = true;
let vSaveHistory = true;
let vAddToBackHistory = true;

function getDef (id, lexicon, word, excludeFromHistory, pushToBackHistory) {
  const skipCache = 0;
  const addWordLinks = 0;

  if (excludeFromHistory) { vSaveHistory = false; } else { vSaveHistory = true; }

  if (pushToBackHistory) { vAddToBackHistory = true; } else { vAddToBackHistory = false; }

  // the random number id needed for ie--it would ask for the same page twice
  const url = 'item?id=' + id + '&lexicon=' + lexicon + '&skipcache=' + skipCache + '&addwordlinks=' + addWordLinks + '&x=' + Math.random();
  // console.log("get def: " + url);

  if (!useDefCache || !defCheckCache(lexicon, id)) {
    loadXMLDoc(url);
  }
  // document.getElementById("lsjdef").innerHTML = "<center>Loading...</center>";
}

function getDefFromWordid (wordid, lexicon, word, excludeFromHistory, pushToBackHistory) {
  const skipCache = 0;
  const addWordLinks = 0;

  if (excludeFromHistory) { vSaveHistory = false; } else { vSaveHistory = true; }

  if (pushToBackHistory) { vAddToBackHistory = true; } else { vAddToBackHistory = false; }

  // the random number id needed for ie--it would ask for the same page twice
  const url = 'item?wordid=' + wordid + '&lexicon=' + lexicon + '&skipcache=' + skipCache + '&addwordlinks=' + addWordLinks + '&x=' + Math.random();

  // if (!useDefCache || !defCheckCache(lexicon, id))
  loadXMLDoc(encodeURI(url));

  // document.getElementById("lsjdef").innerHTML = "<center>Loading...</center>";
}

// for saving history to database
let lastId = -1;
function setWord (json, status) {
  let data;
  // str = resp2;
  // alert(json);
  try {
    if (typeof JSON !== 'undefined') {
      data = JSON.parse(json);
    } else {
      return; // data = eval("(" + json + ")");
    }
  } catch (e) {
    // if (debug) console.log(e.message + "\n" + json);
    return;
  };

  if (!data) {
    return;
  }

  const con = document.getElementById('lsjdef');

  if (data.errorMesg) {
    con.innerHTML = "<div id='lsj222' style='padding:40px 18px;text-align:center;'>" + data.errorMesg + '</div>';
    return;
  }

  const def = data.def;
  const lexicon = data.lexicon;
  const id = data.word_id;
  const word = data.word.replace(/[0-9]/g, ''); // strip number, if any, from end of string
  const wordid = data.word;
  const lemma = data.lemma;
  let pps = data.principalParts;

  document.title = word;

  pps = (pps && pps.length > 0) ? pps : '';

  const perseusLink = "<a href='http://www.perseus.tufts.edu/hopper/text.jsp?doc=Perseus:text:";

  let attr = "<br/><br/><div id='attrib' style='text-align:center;'>";
  if (lexicon === 'lsj') {
    attr += perseusLink + "1999.04.0057' class='attrlink'>Liddell, Scott, and Jones</a> ";
    attr += perseusLink + '1999.04.0057%3Aentry%3D';
  } else if (lexicon && lexicon === 'slater') {
    attr += perseusLink + "1999.04.0072' class='attrlink'>Slater's <i>Lexicon to Pindar</i></a> ";
    attr += perseusLink + '1999.04.0072%3Aentry%3D';
  } else if (lexicon && lexicon === 'ls') {
    attr += perseusLink + "1999.04.0059' class='attrlink'>Lewis and Short</a> ";
    attr += perseusLink + '1999.04.0059%3Aentry%3D';
  }
  attr += escape(lemma);
  attr += "' class='attrlink'>entry</a> courtesy of the<br/>";
  attr += "<a href='http://www.perseus.tufts.edu' class='attrlink'>Perseus Digital Library</a>";
  attr += '</div>';
  // attr += "</div>";

  con.innerHTML = "<div id='lsj222'    style='padding:10px 18px;'><div style='font-size:20pt;margin-bottom:16px;'>" + word + "</div><div style='margin-bottom:24px;'>" + pps + '</div>' + def + attr + '</div>'; // the firstChild is the CDATA node
  con.scrollTo(0, 0);

  if (useDefCache) {
    defAddResultToCache(lexicon, id, json);
  }

  if (vSaveHistory) {
    lastId = id;
    // setTimeout("saveHistory('" + lexicon + "'," + id + ", '" + word + "')", 1500);
    setTimeout(function () {
      saveHistory(lexicon, id, word);
    }, 1500);
  }

  if (vAddToBackHistory) {
    if (window.history && typeof (window.history.pushState) === 'function') {
      // add lexicon and word to path
      if (setURLToWordId) {
        window.history.pushState([id, lexicon], wordid, getPathBeforeLexicon(window.location.pathname) + lexicon + '/' + wordid);
      }
    }
  }
}

function getPathBeforeLexicon (loc) {
  // get path before any lsj/ls/slater; this makes it work on subdirectories
  let phPath = '';
  const a = loc.indexOf('/lsj');
  if (a > -1) {
    phPath = loc.substring(0, a) + '/';
  } else {
    const a = loc.indexOf('/ls');
    if (a > -1) {
      phPath = loc.substring(0, a) + '/';
    } else {
      const a = loc.indexOf('/slater');
      if (a > -1) {
        phPath = loc.substring(0, a) + '/';
      }
    }
  }
  // console.log(phPath);
  return phPath;
}

// function makeQueryString (paramsObj) {
//   let json = '{';

//   for (prop in paramsObj) { json += '"' + prop + '":"' + paramsObj[prop] + '",'; }

//   json = json.replace(/[,]+$/, ''); // trim trailing comma
//   json += '}';

//   return json;
// }

function supportsHTML5Storage () {
  try {
    return 'localStorage' in window && window.localStorage !== null;
  } catch (e) {
    return false;
  }
}

function saveHistory (lexicon, id, word) {
  // alert(lexicon + ", " + id);
  if (id === lastId) {
    if (vIsLoggedIn) {
      const params = {};
      params.userid = 1;
      params.lexicon = lex[0];
      // const q = makeQueryString(params);
      // alert(q);
      const query = '{"id":' + id + ',"lex":"' + lexicon + '","user":1}';
      const url = 'saveHistory.php?query=' + query;
      // alert(url);
      loadXMLDoc(url);
    } else if (supportsHTML5Storage()) {
      let lexi = 0;
      if (lexicon === lex[0]) { lexi = 0; } else if (lexicon === lex[1]) { lexi = 1; } else { lexi = 2; }

      const hist = window.localStorage.getItem('history2');
      let tree = null;
      if (hist && hist.length > 0) {
        tree = JSON.parse(hist);
      } else {
        tree = {
          error: '',
          wtprefix: 'test4',
          container: 'test4Container',
          requestTime: '1427555297518',
          selectId: '-1',
          page: '0',
          lastPage: '1',
          lastPageUp: '1',
          scroll: '',
          query: '',
          arrOptions: []
        };
      }

      if (tree.arrOptions.length < 1 || id !== tree.arrOptions[0][1]) {
        tree.arrOptions.splice(0, 0, [word, id, lexi]);
      }

      const max = 500;
      if (tree.arrOptions.length > max) {
        const toRemove = tree.arrOptions.length - max;
        tree.arrOptions.splice(tree.arrOptions.length - toRemove, toRemove);
      }

      const h = JSON.stringify(tree);

      window.localStorage.setItem('history2', h);
      if (w4) {
        w4.refreshWithRows(h);
      }
    }
  }
}

function refreshHistory () {
  if (typeof w4 !== 'undefined' && w4) {
    w4.refresh();
  }
}

function loadXMLDoc (url) {
  microAjax({
    url,
    method: 'GET',
    success: setWord,
    warning: null,
    error: null
  });
}

function defCheckCache (lexicon, queryKey) {
  queryKey = lexicon + queryKey;
  if (defCache && defCache[queryKey]) {
    // alert("here");
    setWord(defCache[queryKey].str);
    return true;
  } else {
    return false; // not cached, request it
  }
}

function defAddResultToCache (lexicon, queryKey, str) {
  queryKey = lexicon + queryKey;

  // if this query isn't in the cache
  if (!defCache[queryKey]) {
    // if we're at the cacheLimit remove the oldest item
    // use cacheLength because assoc arrays have no length property and we don't want to have to count them each time
    if (defCacheLimit && defCacheLength >= defCacheLimit) {
      let prev = null;
      let x;
      for (x in defCache) {
        if (!defCache.hasOwnProperty(x)) { continue; }

        if (prev == null || defCache[x].time < defCache[prev].time) { prev = x; }
      }
      if (prev) {
        // console.log("delete");
        defCacheLength--;
        delete defCache[prev];
      }
    }
    defCacheLength++;
    defCache[queryKey] = [];
    defCache[queryKey].str = str;
    defCache[queryKey].time = new Date().getTime();
  } else {
    // if it is in the cache, update the timestamp
    defCache[queryKey].time = new Date().getTime();
  }
}
