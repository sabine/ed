// Array.from polyfill
if (!Array.from) {
  Array.from = (function () {
    var toStr = Object.prototype.toString;
    var isCallable = function (fn) {
      return typeof fn === 'function' || toStr.call(fn) === '[object Function]';
    };
    var toInteger = function (value) {
      var number = Number(value);
      if (isNaN(number)) { return 0; }
      if (number === 0 || !isFinite(number)) { return number; }
      return (number > 0 ? 1 : -1) * Math.floor(Math.abs(number));
    };
    var maxSafeInteger = Math.pow(2, 53) - 1;
    var toLength = function (value) {
      var len = toInteger(value);
      return Math.min(Math.max(len, 0), maxSafeInteger);
    };

    // The length property of the from method is 1.
    return function from(arrayLike/*, mapFn, thisArg */) {
      // 1. Let C be the this value.
      var C = this;

      // 2. Let items be ToObject(arrayLike).
      var items = Object(arrayLike);

      // 3. ReturnIfAbrupt(items).
      if (arrayLike == null) {
        throw new TypeError("Array.from requires an array-like object - not null or undefined");
      }

      // 4. If mapfn is undefined, then let mapping be false.
      var mapFn = arguments.length > 1 ? arguments[1] : void undefined;
      var T;
      if (typeof mapFn !== 'undefined') {
        // 5. else
        // 5. a If IsCallable(mapfn) is false, throw a TypeError exception.
        if (!isCallable(mapFn)) {
          throw new TypeError('Array.from: when provided, the second argument must be a function');
        }

        // 5. b. If thisArg was supplied, let T be thisArg; else let T be undefined.
        if (arguments.length > 2) {
          T = arguments[2];
        }
      }

      // 10. Let lenValue be Get(items, "length").
      // 11. Let len be ToLength(lenValue).
      var len = toLength(items.length);

      // 13. If IsConstructor(C) is true, then
      // 13. a. Let A be the result of calling the [[Construct]] internal method of C with an argument list containing the single item len.
      // 14. a. Else, Let A be ArrayCreate(len).
      var A = isCallable(C) ? Object(new C(len)) : new Array(len);

      // 16. Let k be 0.
      var k = 0;
      // 17. Repeat, while k < len… (also steps a - h)
      var kValue;
      while (k < len) {
        kValue = items[k];
        if (mapFn) {
          A[k] = typeof T === 'undefined' ? mapFn(kValue, k) : mapFn.call(T, kValue, k);
        } else {
          A[k] = kValue;
        }
        k += 1;
      }
      // 18. Let putStatus be Put(A, "length", len, true).
      A.length = len;
      // 20. Return A.
      return A;
    };
  }());
}   


var el = domvm.defineElement;
var vw = domvm.defineView;

window.updateValue = function (fieldname, value) {
  window.data[fieldname] = value;
  console.log(["updateValue", fieldname, value, window.data]);
};

window.reloadCss = function reloadCss()
  {
    var links = document.head.getElementsByTagName("link");
    for (var cl in links)
    {
        var link = links[cl];
        if (link.rel === "stylesheet")
            link.href += "";
    }
};
/*
function () {
  console.log("trying to reload CSS...");
  var xmlhttp = new XMLHttpRequest();
  xmlhttp.onreadystatechange = function() {
    if (this.readyState == 4 && this.status == 200) {
      css = document.getElementById("css");
      console.log(css);
      css.textContent = this.responseText;
    }
  };
  xmlhttp.open('GET', '/reload-css');
  xmlhttp.send();
};
*/

window.save = function () {
  console.log("trying to save...");
  external.invoke(JSON.stringify({'cmd': 'SavePage', 'data': window.data}));
};

window.returnToProject = function() {
  console.log("going back to ProjectPage...");
  external.invoke(JSON.stringify({'cmd': 'ReturnToProject'}));
}


document.addEventListener("DOMContentLoaded", function () {
  console.log(document.body);
        
  var div = document.createElement("div");
  div.id = "wrap";
  // Move the body's children into this wrapper
  div.style = "width:80%;position:relative;";
  while (document.body.firstChild)
  {
      div.appendChild(document.body.firstChild);
  }
  // Append the wrapper to the body
  document.body.appendChild(div);
  
  var menu = document.createElement("div");
  document.body.insertBefore(menu, document.body.firstChild); 
  
  function renderInput(fieldname, displayText) {
      return el("div", [
          el("label", displayText)
          , el("input", {type: "text", value:window.data[fieldname], onchange: function (ev) {
              window.updateValue(fieldname, ev.target.value);
          }})
      ]);
  }
  
  function renderTextarea(fieldname, displayText) {
      return el("div", [
          el("label", displayText)
          , el("textarea", {onchange: function (ev) {
              window.updateValue(fieldname, ev.target.value);
          }}, window.data[fieldname])
      ]);
  }
  
  var View = function () {
      return function render (vm, menuData) {
          console.log(["render", data]);
          
          return el("div", {style: "background: #444; padding:12px; z-index:99999; color: gainsboro; position: fixed; "+(!menuData.minimized?"left":"right")+":0; bottom:0; transition: 0.5s; width:"+(menuData.minimized?"20%":"70%")}, [
              el("span", {style: "float:right", onclick: function () {
                  console.log("minimized");
                  menuData.minimized = !menuData.minimized;
                  vm.redraw();
                  }}, "O")
              //, el("span", window.file.name)
              , el("br")
              , el("a", {href: "#", onclick: returnToProject}, "Zurück")
              , el("br")
              //, el("a", {href: "#", onclick: showPreview}, "Vorschau")
              //, el("br")
              , el("a", {href: "#", onclick: save}, "Speichern")
              
              , el("br")
              , el("p", "Metadaten")
              , renderInput("meta_title", "Titel")
              , renderTextarea("meta_description", "Beschreibung")
          ]);
      }
  };
   
  var view = domvm.createView(View, {minimized: true}).mount(menu);
  
  console.log("view created");
});