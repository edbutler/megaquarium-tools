; module for loading game data
#lang racket

(provide
  read-json-file
  read-tanks-from-file
  read-species-from-file
  read-save-from-file)

(require
  json
  "../lift.rkt"
  "../core.rkt")

(module+ test
  (require rackunit))

(define (jsexpr-ref obj . keys)
  (match keys
   [(cons head tail)
    (define next
      (cond
       [(number? head) (list-ref obj head)]
       [else (hash-ref obj head)]))
    (apply jsexpr-ref next tail)]
   ['() obj]))

(module+ test
  (test-case "jsexpr-ref can reference nested json"
    (define js (string->jsexpr "[0,{\"key\":[\"hamster\"]}]"))
    (check-equal? (jsexpr-ref js 1 'key 0) "hamster")))

; find all the subobjects of this json-like with the given key
(define (jsexpr-search obj key)
  (cond
   [(list? obj)
    (append-map (λ (x) (jsexpr-search x key)) obj)]
   [(hash? obj)
    (define rest (jsexpr-search (hash-values obj) key))
    (if (hash-has-key? obj key)
      (cons (hash-ref obj key) rest)
      rest)]
   [else '()]))

(module+ test
  (test-case "jsexpr-search finds all keys"
    (define str #<<JSON
      {
        "key1":[1,{"key2":"pika"}],
        "key2":"hamster",
        "key3":{"key1":true},
        "key4":[[[{"a":"b","key2":"mouse"}]]]
      }
JSON
    )
    (define js (string->jsexpr str))
    (check-equal? (jsexpr-search js 'key1)
                  (list (list 1 (hasheq 'key2 "pika"))
                        #t))
    (check-equal? (jsexpr-search js 'key2)
                  '("hamster" "mouse" "pika"))
    (check-equal? (jsexpr-search js 'otherkey) '())))

; takes care of bad stuff like trailing commas
(define (read-json-file filename)
  (define text (file->string filename))
  ; racket's json parser is strict, so first remove trailing commas, comments, multiline strings
  (set! text (regexp-replace* #rx"\"map\":\".*?\"" text "\"map\":\"\"\n"))
  (set! text (regexp-replace* #rx"//.*?\n" text "\n"))
  (set! text (regexp-replace* #rx"/\\*.*?\\*/" text ""))
  (set! text (regexp-replace* #rx",([\r\n |\t]*})" text "\\1"))
  (set! text (regexp-replace* #rx",([\r\n |\t]*])" text "\\1"))
  ;(display-to-file text "test.json" #:exists 'replace) ; for debugging when things go wrong
  (string->jsexpr text))

(define (read-tank-kind jsexpr)
  (define id (string->symbol (hash-ref jsexpr 'id)))
  (define density (jsexpr-ref jsexpr 'tank 'volumePerTile))
  (define (dim pr) (cons (hash-ref pr 'm) (hash-ref pr 'n)))
  (define min-dim (dim (jsexpr-ref jsexpr 'multisize 'minSize)))
  (define max-dim (dim (jsexpr-ref jsexpr 'multisize 'baseSize)))

  (make-tank-kind
    id
    #:min min-dim
    #:max max-dim
    #:density density
    #:rounded? #f))

(define (read-tanks-from-file filename)
  (define jdata (read-json-file filename))
  (map read-tank-kind (hash-ref jdata 'objects)))

; number?, symbol? -> predator?
; the game mechanics for eating, based on final size
(define (make-predator final-size jkey)
  ; number from https://steamcommunity.com/app/600480/discussions/0/3276824488724294545/
  (define num (exact-floor (* 0.4 final-size)))
  ; drop the "Eater" from the key
  (define type (string->symbol (string-trim (symbol->string jkey) "Eater")))

  ; certain types have no size constraint
  (define size
    (match type
     ['fish num]
     ['crustacean num]
     [_ #f]))

  (predator type size))

; localization = (hashof symbol? string?)
; class_ = 'fish or 'coral
(define (read-species class_ jval)
  (define janimal (hash-ref jval 'animal))
  (define jstats (hash-ref janimal 'stats))
  (define junlock (hash-ref jval 'unlockable))
  (define (when-has-stat key elem) (maybe-singleton (hash-has-key? jstats key) elem))
  (define (when-has-stat/value key f key-2)
    (maybe-singleton (hash-has-key? jstats key) (f (hash-ref (hash-ref jstats key) key-2))))
  (define stages
    (map
      (λ (j)
        (species-stage (hash-ref j 'size 0)
                       (hash-ref j 'growthTime #f)))
      (hash-ref janimal 'stages)))
  (define final-size (species-stage-size (last stages)))
  (species
    (string->symbol (hash-ref jval 'id))
    class_
    (string->symbol (list-ref (hash-ref jval 'tags) 1))
    (size stages #f) ; TODO armored
    (environment
      (cond
       [(hash-has-key? jstats 'isTropical) warm-water]
       [(hash-has-key? jstats 'isColdwater) cold-water]
       [else (error "unknown temperature")])
      (hash-ref (hash-ref jstats 'waterQuality) 'value))
    (let ([jeats (hash-ref jstats 'eats #f)])
      (cond
       [jeats
        (food (string->symbol (hash-ref jeats 'item))
              (add1 (hash-ref jeats 'daysBetweenFeed 0)))]
       [(hash-has-key? jstats 'scavenger)
        (scavenger)]
       [else
        (does-not-eat)]))
    (apply
      append
      (list
        (when-has-stat 'bully (bully))
        ))
    (apply
      append
      (list
        ; shoaling
        (when-has-stat/value 'shoaler shoaler 'req)
        ; predation
        (let ([j (hash-ref jstats 'eater #f)])
          (maybe-list j
            (map (curry make-predator final-size) (hash-keys j))))
        ; likes/dislikes
        (when-has-stat 'activeSwimmer (active-swimmer 6))
        (when-has-stat 'needsRounded (rounded-tank))
        (when-has-stat 'dislikesConspecifics (dislikes-conspecifics))
        (when-has-stat 'dislikesCongeners (dislikes-congeners))
        (when-has-stat 'congenersOnly (only-congeners))
        (when-has-stat 'dislikesFoodCompetitors (dislikes-food-competitors))
        (when-has-stat 'dislikesLights (dislikes-light))
        (when-has-stat 'wimp (wimp))
        (when-has-stat/value 'light requires-light 'value)
        ))
    (unlockable (hash-ref junlock 'availableLevel))))

(define (read-species-from-file #:animals animal-filename #:corals coral-filename)
  (append-map
    (λ (fname cls)
      (let ([jdata (read-json-file fname)])
        (map (curry read-species cls) (hash-ref jdata 'objects))))
    (list animal-filename coral-filename)
    (list 'fish 'coral)))

; (listof species?) -> json? -> aquarium?
(define (read-save-data all-species jdata)
  (define j-object-list (hash-ref jdata 'objects))

  ; extract animals, paired with tank ids
  ; (listof (pairof animal? integer?))
  (define animals
    (filter-map
      (λ (jobj)
        (cond
         [(and (hash-has-key? jobj 'animal) (hash-ref jobj 'inGameWorld #f))
          (define spec
            (let ([spec-id (string->symbol (hash-ref jobj 'specId))])
              spec-id))
          (cons
            (animal (hash-ref jobj 'uid)
                    spec)
            (hash-ref (hash-ref jobj 'hosting) 'host))]
         [else #f]))
      j-object-list))

  (define tanks
    (filter-map
      (λ (jobj)
        (cond
         [(and (hash-has-key? jobj 'tank) (hash-ref jobj 'inGameWorld #f))
          (make-tank
            (hash-ref jobj 'uid)
            #:name (hash-ref jobj 'name)
            #:size #f
            #:environment #f
            #:lighting #f)]
         [else #f]))
      j-object-list))

  (define max-rank (jsexpr-ref jdata 'gameParameters 'maxRank))

  (define j-unlocks (jsexpr-ref jdata 'playerData 'unlockableManager))

  (define j-objectives (jsexpr-ref jdata 'playerData 'scenario 'sections))

  (define all-animal-ids
    (map species-id
         (filter (λ (s) (<= (unlockable-rank (species-unlockable s)) max-rank))
                 all-species)))

  (define (read-spec-list jobj key)
    (map string->symbol (hash-ref jobj key)))

  (define available
    (set-intersect all-animal-ids (read-spec-list j-unlocks 'unlockedSpecs)))
  (define researchable
    (set-subtract all-animal-ids (set-union available (read-spec-list j-unlocks 'excludedSpecs))))

  (define j-merchants (jsexpr-search (hash-ref jdata 'playerData) 'merchant))

  (define acquirable
    (map (λ (x) (cons (string->symbol (hash-ref x 'id))
                      (hash-ref x 'quantity)))
         j-merchants))

  (define objectives
    (filter-map
      (λ (o)
        (define goals (hash-ref o 'objectives))
        (define obj
          (findf (λ (g) (equal? (hash-ref g 'id) "tankWithXAnimal")) goals))
        (define condition
          (cond
           [obj
            (define cond1 (jsexpr-ref obj 'conditions 0))
            ;(displayln cond1)
            (define val (jsexpr-ref cond1 'tank 'hostsMany 0))
            (define num (hash-ref val 'quantity))
            ; not sure if this is necessary seems to be true on all relevant conditions
            (define ds (hash-ref val 'differentSpec #f))
            (cond
             [(hash-has-key? val 'id) (species-condition (hash-ref val 'id) num)]
             [(and ds (hash-ref val 'shoaler #f)) (restriction-condition 'shoaler num)]
             [(and ds (hash-ref val 'wimp #f)) (restriction-condition 'wimp num)]
             [(and ds (hash-ref val 'activeSwimmer #f)) (restriction-condition 'active-swimmer num)]
             [(and ds (hash-has-key? val 'tag) (not (hash-has-key? val 'insertOverride)))
              (type-condition (string->symbol (hash-ref val 'tag)) num)]
             [else obj])]
           [else
            ;(displayln goals)
           #f]))
        (and condition (objective condition)))
      j-objectives))

  (aquarium
    (map
      (λ (tnk)
        (cons
          tnk
          (filter-map (λ (pr) (and (= (tank-id tnk) (cdr pr)) (car pr))) animals)))
      tanks)
    (market available researchable acquirable)
    objectives))

(define (read-save-from-file filename #:species species-list)
  (let ([jdata (read-json-file filename)])
    (read-save-data species-list jdata)))