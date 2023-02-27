/**
 * Copyright 2006-2018 by Jeremy March.  All rights reserved.
 */
var reqCOM;
var defCache = new Array();
var defCacheLength = 0;
var defCacheLimit = 500;
var useDefCache = true;
var vSaveHistory = true;
var vAddToBackHistory = true;

function getDef(id, lexicon, word, excludeFromHistory, pushToBackHistory) {
  var skipCache = 0;
  var addWordLinks = 0;

  if (excludeFromHistory)
    vSaveHistory = false;
  else
    vSaveHistory = true;

  if (pushToBackHistory)
    vAddToBackHistory = true;
  else
    vAddToBackHistory = false;

  //the random number id needed for ie--it would ask for the same page twice
  var url = 'item?id=' + id + '&lexicon=' + lexicon + '&skipcache=' + skipCache + '&addwordlinks=' + addWordLinks + '&x=' + Math.random();
  //console.log("get def: " + url);

  if (!useDefCache || !defCheckCache(lexicon, id)) {
    loadXMLDoc(url);
  }
  else {
    return;
  }

  //document.getElementById("lsjdef").innerHTML = "<center>Loading...</center>";
}

function getDefFromWordid(wordid, lexicon, word, excludeFromHistory, pushToBackHistory) {
  //alert(wordid + ",2 " + lexicon);

  var skipCache = 0;
  var addWordLinks = 0;

  if (excludeFromHistory)
    vSaveHistory = false;
  else
    vSaveHistory = true;

  if (pushToBackHistory)
    vAddToBackHistory = true;
  else
    vAddToBackHistory = false;

  //the random number id needed for ie--it would ask for the same page twice
  var url = 'item?wordid=' + wordid + '&lexicon=' + lexicon + '&skipcache=' + skipCache + '&addwordlinks=' + addWordLinks + '&x=' + Math.random();
  //alert(url);

  //if (!useDefCache || !defCheckCache(lexicon, id))
  loadXMLDoc(encodeURI(url));

  //else
  //		return;

  //document.getElementById("lsjdef").innerHTML = "<center>Loading...</center>";
}

//for saving history to database
var lastLex = "";
var lastId = -1;
var debug = false;

function setWord(json, status) {
  var data;
  //str = resp2;
  //alert(json);
  try {
    if (typeof JSON != "undefined") {
      data = JSON.parse(json);
    } else {
      return; //data = eval("(" + json + ")");
    }
  } catch (e) {
    //if (debug) console.log(e.message + "\n" + json);
    return;
  };

  if (!data) {
    return;
  }

  var con = document.getElementById("lsjdef");

  if (data.errorMesg) {
    con.innerHTML = "<div id='lsj222' style='padding:40px 18px;text-align:center;'>" + data.errorMesg + "</div>";
    return;
  }

  var def = data.def;
  var lexicon = data.lexicon;
  var id = data.word_id;
  var word = data.word.replace(/[0-9]/g, ''); //strip number, if any, from end of string
  var wordid = data.word;
  var lemma = data.lemma;
  var pps = data.principalParts;

  document.title = word;

  pps = (pps && pps.length > 0) ? pps : "";

  if (lexicon) {
    lexicon = lexicon;
  }

  var perseusLink = "<a href='http://www.perseus.tufts.edu/hopper/text.jsp?doc=Perseus:text:";

  var attr = "<br/><br/><div id='attrib' style='text-align:center;'>";
  if (lexicon && lexicon == "lsj") {
    attr += perseusLink + "1999.04.0057' class='attrlink'>Liddell, Scott, and Jones</a> ";
    attr += perseusLink + "1999.04.0057%3Aentry%3D";
  } else if (lexicon && lexicon == "slater") {
    attr += perseusLink + "1999.04.0072' class='attrlink'>Slater's <i>Lexicon to Pindar</i></a> ";
    attr += perseusLink + "1999.04.0072%3Aentry%3D";
  } else if (lexicon && lexicon == "ls") {
    attr += perseusLink + "1999.04.0059' class='attrlink'>Lewis and Short</a> ";
    attr += perseusLink + "1999.04.0059%3Aentry%3D";
  }
  attr += escape(lemma);
  attr += "' class='attrlink'>entry</a> courtesy of the<br/>";
  attr += "<a href='http://www.perseus.tufts.edu' class='attrlink'>Perseus Digital Library</a>";
  attr += "</div>";
  //attr += "</div>";

  con.innerHTML = "<div id='lsj222'  style='padding:10px 18px;'><div style='font-size:20pt;margin-bottom:16px;'>" + word + "</div><div style='margin-bottom:24px;'>" + pps + "</div>" + def + attr + "</div>"; //the firstChild is the CDATA node
  con.scrollTo(0,0);
  
  if (useDefCache) {
    defAddResultToCache(lexicon, id, json);
  }

  if (vSaveHistory) {
    lastId = id;
    setTimeout("saveHistory('" + lexicon + "'," + id + ", '" + word + "')", 1500);
  }
  if (vAddToBackHistory) {

    if (history && typeof(history.pushState) == "function") {
      
	    var ee = window.location.pathname;
      
      //add lexicon and word to path
      if (setURLToWordId) {
        history.pushState([id, lexicon], wordid, getPathBeforeLexicon(ee) + lexicon + '/' + wordid);
      }
    }
  }
}

function getPathBeforeLexicon(ee) {
      //get path before any lsj/ls/slater; this makes it work on subdirectories
      var phPath = "";
      var a = ee.indexOf("/lsj");
      if (a > -1) {
        phPath = ee.substring(0, a) + "/";
      }
      else {
        var a = ee.indexOf("/ls");
        if (a > -1) {
          phPath = ee.substring(0, a) + "/";
        }
        else {
          var a = ee.indexOf("/slater");
          if (a > -1) {
            phPath = ee.substring(0, a) + "/";
          }        
        }
      }
      //console.log(phPath);
      return phPath;
}

function makeQueryString(paramsObj) {
  var json = "{";

  for (prop in paramsObj)
    json += '"' + prop + '":"' + paramsObj[prop] + '",';

  json = json.replace(/[,]+$/, ""); //trim trailing comma
  json += "}";

  return json;
}

function supports_html5_storage() {
  try {
    return 'localStorage' in window && window['localStorage'] !== null;
  } catch (e) {
    return false;
  }
}

function saveHistory(lexicon, id, word) {
  //alert(lexicon + ", " + id);
  if (id == lastId) {
    if (vIsLoggedIn) {
      var params = new Object();
      params.userid = 1;
      params.lexicon = lex[0];
      var q = makeQueryString(params);
      //alert(q);
      var query = '{"id":' + id + ',"lex":"' + lexicon + '","user":1}';
      var url = "saveHistory.php?query=" + query;
      //alert(url);
      loadXMLDoc(url);
    } else if (supports_html5_storage()) {
      var lexi = 0;

      if (lexicon == lex[0])
        lexi = 0;
      else if (lexicon == lex[1])
        lexi = 1;
      else
        lexi = 2;

      var h = localStorage.getItem("history2");
      if (h && h.length > 0) {
        var tree = JSON.parse(h);
      } else {
        var tree = {
          "error": "",
          "wtprefix": "test4",
          "container": "test4Container",
          "requestTime": "1427555297518",
          "selectId": "-1",
          "page": "0",
          "lastPage": "1",
          "lastPageUp": "1",
          "scroll": "",
          "query": "",
          "arrOptions": []
        };
      }

      if (tree.arrOptions.length < 1 || id != tree.arrOptions[0][1]) {
        tree.arrOptions.splice(0, 0, [word, id, lexi]);
      }

      var max = 500;
      if (tree.arrOptions.length > max) {
        var toRemove = tree.arrOptions.length - max;
        tree.arrOptions.splice(tree.arrOptions.length - toRemove, toRemove);
      }

      var h = JSON.stringify(tree);

      localStorage.setItem("history2", h);
      if (w4)
        w4.refreshWithRows(h);
    }
  }
}

function refreshHistory() {
  if (typeof w4 != "undefined" && w4 && tagsOpen)
    w4.refresh();
}


function loadXMLDoc(url) {

    microAjax({
      url: url,
      method: "GET",
      success: setWord,
      warning: null,
      error: null
    });

/*
  $.ajax({
    url: url,
    type: "GET",
    data: {},
    scriptCharset: "utf-8",
    contentType: "application/x-www-form-urlencoded; charset=UTF-8",
    dataType: "text", //JSON automatically deserializes json
    success: function(data, textStatus, jqXHR) {

      //if (data.indexOf("setWord") != -1)
      setWord(data, textStatus);
      //else
      //	refreshHistory();

      //eval(method + '(response)');
    },
    error: function(response) {
      loadXMLDoc(url); //redo request on error
    }
  });
  */
}

function defview(node) {
  this.node = mode;
  this.requestDef = null;
  this.onDefReceived = null;
}

function defCheckCache(lexicon, queryKey) {
  queryKey = lexicon + queryKey;
  if (defCache && defCache[queryKey]) {
    //alert("here");
    setWord(defCache[queryKey].str);
    return true;
  } else {
    return false; //not cached, request it
  }
}

function defAddResultToCache(lexicon, queryKey, str) {
  queryKey = lexicon + queryKey;

  //if this query isn't in the cache
  if (!defCache[queryKey]) {
    //if we're at the cacheLimit remove the oldest item
    //use cacheLength because assoc arrays have no length property and we don't want to have to count them each time
    if (defCacheLimit && defCacheLength >= defCacheLimit) {
      var prev = null;
      for (var x in defCache) {
        if (!defCache.hasOwnProperty(x))
          continue;

        if (prev == null || defCache[x].time < defCache[prev].time)
          prev = x;
      }
      if (prev) {
        //alert("delete");
        defCacheLength--;
        delete defCache[prev];
      }
    }
    defCacheLength++;
    defCache[queryKey] = new Array();
    defCache[queryKey].str = str;
    defCache[queryKey].time = new Date().getTime();
  } else {
    //if it is in the cache, update the timestamp
    defCache[queryKey].time = new Date().getTime();
  }
}
