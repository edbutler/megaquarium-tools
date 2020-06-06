#lang errortrace racket

(require
  "cli-check-listing.rkt"
  "cli-extract.rkt"
  "cli-info.rkt"
  "cli-lookup.rkt"
  "types.rkt"
  "localization.rkt"
  "lift.rkt"
  "constraint.rkt"
  "serialize.rkt"
  "display.rkt"
  "data.rkt")

; (listof? species-spec?) -> ...
(define (make-fishes specs #:id-suffix [id-suffix ""])
  (define (make sspec)
    (define spc (species-spec-species sspec))
    (define cnt (species-spec-count sspec))
    (define (name i) (format "~a-~a~a" (species-id spc) i id-suffix))
    (build-list cnt (λ (i) (animal (name i) spc))))
  (append-map make specs))

; constructions a tank that could plausibly fit the given animals (minimal size, etc.)
(define (minimum-viable-tank animals #:name [name #f] #:id [id 1])
  (make-tank
    id
    #:name (or name "tank")
    #:type (tank-info #f #f #f (ormap (λ (a) (ormap rounded-tank? (animal-restrictions a))) animals))
    #:size (max (sum animal-final-size animals)
                (max-by (λ (a) (or (ormap (λ (r) (and (active-swimmer? r) (* (active-swimmer-multiplier r) (animal-final-size a))))
                                          (animal-restrictions a))
                                   0))
                        animals))
    #:environment (environment (environment-temperature (animal-environment (first animals)))
                               (apply max (map (compose1 environment-quality animal-environment) animals)))
    #:lighting (max-by (λ (a) (or (ormap (λ (r) (and (requires-light? r) (requires-light-amount r))) (animal-restrictions a)) 0)) animals)))

(define (format-violation violtn)
  (match violtn
   [(cons (? animal? animl) restr)
    (format "\t~a has unmet requirement ~v\n" (localize-animal default-l10n animl) restr)]
   [(cons subj message)
    (format "\tTank has unmet requirement ~a\n" message)]))

(define (do-check-tank-listing)
  (define argv (current-command-line-arguments))
  (define input-filename (vector-ref argv 0))
  (define data (call-with-input-file input-filename (λ (f) (read-tank-yaml f #:species all-species))))
  (define dom-data
    (map
      (λ (i spec)
        (define animals (make-fishes (tank-spec-contents spec) #:id-suffix (format "-~a" i)))
        (define tnk (minimum-viable-tank animals #:name (tank-spec-name spec) #:id i))
        (cons tnk animals))
      (range (length data))
      data))
  (define dom (make-concrete-domain dom-data))
  (define satisfied? (all-constraints-satisfied? dom))
  (printf "Valid acquarium? ~a\n" satisfied?)
  (cond
   [satisfied?
    (for ([data dom-data])
      (match-define (cons tnk animals) data)
      (printf "~a:\n\tsize: ~a\n\ttemp: ~a\n\tquality: ~a\n"
              (tank-display-name tnk)
              (tank-size tnk)
              (environment-temperature (tank-environment tnk))
              (environment-quality (tank-environment tnk)))
      (when (> (tank-lighting tnk) 0)
        (printf "\tlight level: ~a\n" (tank-lighting tnk)))
      (when (tank-info-rounded? (tank-type tnk))
        (printf "\trounded\n"))
      (define animals-grouped-by-food
        (group-by (compose1 food-type animal-diet)
                  (filter (compose1 food? animal-diet) animals)))
      (for ([lst animals-grouped-by-food])
        (printf "\t~a ~a\n"
                (sum animal-required-food-amount lst)
                (food-type (animal-diet (first lst))))))]
   [else
    (define strs
      (append
        (map format-violation (find-violated-restrictions dom))
        (map format-violation (find-violated-tank-constraints dom))))
    (for ([s (remove-duplicates strs)])
      (printf "~a" s))]))

;(define (do-help)
;  (printf "Usage: program <subcommand> where subcommand is one of:\n")
;  (printf "\t~a\n" (string-join (filter-not (λ (s) (string-prefix? s "-")) (hash-keys subcommands)) " | ")))

;(let* ([argv (current-command-line-arguments)]
;       [subcom (and (> (vector-length argv) 0) (vector-ref argv 0))])
;  (cond
;   [subcom
;    ; remove the subcommand from the args
;    (current-command-line-arguments (vector-drop argv 1))
;    ((hash-ref subcommands subcom))]
;   [else (do-help)]))

; all functions expected to be string?, (listof string?) -> void?
(define subcommands
  (hash "info" info
        "lookup" lookup
        "extract" extract
        "check-listing" check-listing))

(define commands-str
  (string-join (hash-keys subcommands) ", "))

(define command-help
  (format "where <command> is one of { ~a }" commands-str))

(define program-name "cli")

(define (run flags command . args)
  (define fn (hash-ref subcommands command #f))
  (unless fn
    (displayln (format "Unknown command '~a', must be one of { ~a }" command commands-str))
    (exit 1))
  (fn (format "~a ~a" program-name command) args))

(module+ main
  (parse-command-line program-name (current-command-line-arguments)
    `((usage-help ,command-help "and <args> are the command's arguments, and")
      (ps ,(format "Use ~a <command> -h for help on individual commands." program-name)))
    run
    '("command" "args")))
